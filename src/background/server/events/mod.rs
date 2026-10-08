//! Global events transport: a websocket per webview served by the local http server.
//! Events targeted to a single webview still use tauri (`emit_to` / `webview.listen`).

mod connection;
mod hub;
mod protocol;
mod tokens;

use seelen_core::handlers::SelfEventsToken;

use super::http::LOCAL_API_PORT;

pub use connection::events_ws;
pub use hub::EVENTS_HUB;

impl crate::tauri_handlers::Handlers {
    pub fn get_self_token(webview: tauri::WebviewWindow) -> SelfEventsToken {
        SelfEventsToken {
            token: tokens::get_or_create(webview.label()).to_string(),
            endpoint: format!("ws://127.0.0.1:{LOCAL_API_PORT}/events"),
        }
    }
}
