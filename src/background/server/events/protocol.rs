use serde::{Deserialize, Serialize};

/// Messages sent by the webview through the events websocket.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ClientMessage {
    Subscribe { event: String },
    Unsubscribe { event: String },
}

/// Control messages sent to the webview. Event frames are built by hand
/// (see `event_frame`) to embed the already serialized payload without re-serializing it.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ServerMessage {
    /// The subscription is registered, events emitted from now on will be delivered.
    Ack { event: String },
}

/// Builds the `{"type":"event","event":...,"payload":...}` frame from an already serialized payload.
pub fn event_frame(event: &str, payload_json: &str) -> serde_json::Result<String> {
    Ok(format!(
        r#"{{"type":"event","event":{},"payload":{payload_json}}}"#,
        serde_json::to_string(event)?
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_frame_is_valid_json() {
        let frame = event_frame("media::waveform", r#"{"samples":[0.5]}"#).unwrap();
        let value: serde_json::Value = serde_json::from_str(&frame).unwrap();
        assert_eq!(value["type"], "event");
        assert_eq!(value["event"], "media::waveform");
        assert_eq!(value["payload"]["samples"][0], 0.5);
    }

    #[test]
    fn client_messages_parse() {
        let msg: ClientMessage =
            serde_json::from_str(r#"{"type":"subscribe","event":"power-status"}"#).unwrap();
        assert!(matches!(msg, ClientMessage::Subscribe { event } if event == "power-status"));
    }
}
