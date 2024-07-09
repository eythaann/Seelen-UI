use tauri::Manager;

use crate::seelen::get_app_handle;

#[tauri::command]
pub fn request_media_sessions() {
    std::thread::spawn(|| {
        tauri::async_runtime::block_on(async {
            let sessions = super::application::request_media_sessions().await;
            if let Ok(sessions) = sessions {
                let handle = get_app_handle();
                handle
                    .emit("media-sessions", &sessions)
                    .expect("failed to emit");
            }
        });
    });
}
