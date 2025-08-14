use slu_ipc::{messages::SvcAction, ServiceIpc};
use tauri::WebviewWindow;

use crate::{error::Result, widgets::popups::shortcut_registering::REG_SHORTCUT_DATA};

#[tauri::command(async)]
pub async fn request_to_user_input_shortcut(
    window: WebviewWindow,
    callback_event: String,
) -> Result<()> {
    ServiceIpc::send(SvcAction::StartShortcutRegistration).await?;

    let mut data = REG_SHORTCUT_DATA.lock();
    data.response_view_label = Some(window.label().to_string());
    data.response_event = Some(callback_event);
    Ok(())
}
