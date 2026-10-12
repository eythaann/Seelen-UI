use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, state::settings::SessionTimeSound};
use tauri::WebviewWindow;

use crate::{
    app::emit_to_webviews,
    modules::session_time::application::{SessionTimeEvent, SessionTimeManager, play_sound},
    state::application::FULL_STATE,
    widgets::permissions::{WidgetPerm, request_widget_permission},
};

fn get_session_time_manager() -> &'static SessionTimeManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        SessionTimeManager::subscribe(|event| match event {
            SessionTimeEvent::Updated(snapshot) => {
                emit_to_webviews(SeelenEvent::ActiveSessionTimeChanged, snapshot);
            }
        });
    });
    SessionTimeManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_active_session_time() -> seelen_core::system_state::ActiveSessionTime {
        get_session_time_manager().snapshot()
    }

    pub fn get_session_time_audio_cue_sounds() -> Vec<SessionTimeSound> {
        super::application::available_system_sounds()
    }

    pub fn set_session_time_audio_cue_settings(
        webview: WebviewWindow,
        enabled: bool,
        interval_minutes: u32,
        sound: SessionTimeSound,
        custom_path: Option<String>,
        notification_enabled: bool,
        notification_message: String,
    ) -> crate::error::Result<()> {
        request_widget_permission(&webview, WidgetPerm::ModifyConfiguration)?;
        let custom_path = custom_path.map(std::path::PathBuf::from);
        if sound == SessionTimeSound::CustomMp3 {
            let Some(path) = custom_path.as_ref() else {
                return Err("Select an MP3 file for the custom screen-time sound".into());
            };
            if !path.is_file()
                || !path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("mp3"))
            {
                return Err("The custom screen-time sound must be an existing MP3 file".into());
            }
        }
        let custom_path = custom_path.map(|path| path.to_string_lossy().into_owned());

        let mut settings = FULL_STATE.load().settings.clone();
        settings.session_time_audio_cue_enabled = enabled;
        settings.session_time_audio_cue_interval_minutes = interval_minutes.max(1);
        settings.session_time_audio_cue_sound = sound;
        settings.session_time_audio_cue_custom_path = custom_path;
        settings.session_time_notification_enabled = notification_enabled;
        settings.session_time_notification_message =
            notification_message.trim().chars().take(200).collect();
        settings.sanitize()?;
        FULL_STATE.rcu(move |state| {
            let mut state = state.cloned();
            state.settings = settings.clone();
            state
        });
        FULL_STATE.load().write_settings()?;
        crate::backups::application::on_settings_saved();
        Ok(())
    }

    pub fn test_session_time_audio_cue(
        webview: WebviewWindow,
        sound: SessionTimeSound,
        custom_path: Option<String>,
    ) -> crate::error::Result<()> {
        request_widget_permission(&webview, WidgetPerm::ModifyConfiguration)?;
        log::info!("Testing screen-time cue sound: {sound:?}");
        play_sound(sound, custom_path)
    }
}
