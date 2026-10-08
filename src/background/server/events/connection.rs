use std::{sync::Arc, time::Duration};

use salvo::{
    prelude::*,
    websocket::{Message, WebSocket, WebSocketUpgrade},
};
use uuid::Uuid;

use super::{
    hub::{Connection, EVENTS_HUB},
    protocol::{ClientMessage, ServerMessage},
    tokens,
};

const PING_INTERVAL: Duration = Duration::from_secs(30);

/// `GET /events?token=<uuid>`
///
/// Global events channel for the webviews. The token is obtained via the `GetSelfToken`
/// command and identifies the webview, so the client never sends its label.
#[handler]
pub async fn events_ws(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    // never log the query, it contains the token
    let label = req
        .query::<String>("token")
        .and_then(|token| Uuid::parse_str(&token).ok())
        .and_then(|token| tokens::resolve(&token));

    let Some(label) = label else {
        return Err(StatusError::unauthorized());
    };

    WebSocketUpgrade::new()
        .upgrade(req, res, move |ws| handle_connection(ws, label))
        .await
}

async fn handle_connection(mut ws: WebSocket, label: String) {
    let connection = EVENTS_HUB.connect(&label);
    let mut ping = tokio::time::interval(PING_INTERVAL);
    ping.tick().await; // the first tick completes immediately

    loop {
        tokio::select! {
            msg = ws.recv() => {
                let Some(Ok(msg)) = msg else {
                    break;
                };
                if msg.is_close() {
                    break;
                }
                if let Ok(text) = msg.as_str()
                    && handle_client_message(&mut ws, &connection, text).await.is_err()
                {
                    break;
                }
            }
            _ = connection.notified() => {
                if connection.is_closed() || send_pending(&mut ws, &connection).await.is_err() {
                    break;
                }
            }
            _ = ping.tick() => {
                if ws.send(Message::ping(Vec::new())).await.is_err() {
                    break;
                }
            }
        }
    }

    EVENTS_HUB.disconnect(&label, &connection);
    let _ = ws.close().await;
}

async fn handle_client_message(
    ws: &mut WebSocket,
    connection: &Arc<Connection>,
    text: &str,
) -> salvo::Result<()> {
    let message = match serde_json::from_str::<ClientMessage>(text) {
        Ok(message) => message,
        Err(err) => {
            log::debug!("Invalid events client message: {err}");
            return Ok(());
        }
    };

    match message {
        ClientMessage::Subscribe { event } => {
            if !EVENTS_HUB.subscribe(connection, &event) {
                log::debug!("Webview tried to subscribe to unknown event: {event}");
                return Ok(());
            }
            let ack = serde_json::to_string(&ServerMessage::Ack { event })?;
            ws.send(Message::text(ack)).await?;
        }
        ClientMessage::Unsubscribe { event } => {
            EVENTS_HUB.unsubscribe(connection, &event);
        }
    }
    Ok(())
}

/// Sends everything pending. Events published while awaiting the socket are coalesced
/// by the hub and sent on the next wake up (the notify permit is kept).
async fn send_pending(ws: &mut WebSocket, connection: &Connection) -> salvo::Result<()> {
    for pending in connection.drain() {
        ws.send(Message::text(pending.frame.to_string())).await?;
    }
    Ok(())
}
