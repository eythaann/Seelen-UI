use std::path::Path;

use seelen_core::state::{CssStyles, Dialog, DialogContent, Settings};

use crate::{
    app::SeelenUI,
    error::{Result, ResultLogExt},
    resources::{NATIVE_WALLPAPER_ID, ResourceManager},
    state::domain::StateEvent,
    utils::constants::SEELEN_COMMON,
    widgets::{
        manager::WIDGET_MANAGER, trigger_dialog_backend, window_manager::state_v2::WM_STATE,
    },
};

use super::{AppSettings, StateManager};

impl AppSettings {
    pub(super) fn emit_settings(&self) -> Result<()> {
        StateManager::send(StateEvent::SettingsChanged {
            settings: Box::new(self.settings.clone()),
        });
        SeelenUI::on_settings_change(self)?;
        WIDGET_MANAGER.reconcile().log_error();
        WM_STATE.lock().on_settings_changed();
        Ok(())
    }

    pub(super) fn read_settings() -> Settings {
        let path = SEELEN_COMMON.settings_path();
        if path.exists() {
            match Settings::load(path) {
                Ok(settings) => return settings,
                Err(err) => {
                    log::error!("Failed to read settings: {err}");
                    show_corrupted_state_to_user(SEELEN_COMMON.settings_path());
                }
            }
        }
        Settings::default()
    }

    /// Resources id changed for remote/downloaded resources.
    pub(super) fn migration_v2_5_0(&mut self, resources: &ResourceManager) -> Result<()> {
        resources.themes.iter_sync(|new_id, theme| {
            let Some(remote) = &theme.metadata.internal.remote else {
                return true;
            };

            let Some(old_id) = remote.deprecated_id.clone() else {
                return true;
            };
            let old_id = old_id.into();

            if let Some(config) = self.settings.by_theme.remove(&old_id) {
                self.settings.by_theme.insert(new_id.clone(), config);
            };

            for id in &mut self.settings.active_themes {
                if id == &old_id {
                    *id = new_id.clone();
                }
            }
            true
        });

        resources.wallpapers.iter_sync(|new_id, wallpaper| {
            let Some(remote) = &wallpaper.metadata.internal.remote else {
                return true;
            };

            let Some(old_id) = remote.deprecated_id.clone() else {
                return true;
            };
            let old_id = old_id.into();

            if let Some(config) = self.settings.by_wallpaper.remove(&old_id) {
                self.settings.by_wallpaper.insert(new_id.clone(), config);
            };

            for collection in &mut self.settings.wallpaper_collections {
                for id in &mut collection.wallpapers {
                    if id == &old_id {
                        *id = new_id.clone();
                    }
                }
            }
            true
        });

        resources.widgets.iter_sync(|k, v| {
            let Some(remote) = &v.metadata.internal.remote else {
                return true;
            };

            let Some(old_id) = remote.deprecated_id.clone() else {
                return true;
            };
            let old_id = old_id.into();

            if let Some(config) = self.settings.by_widget.others.remove(&old_id) {
                self.settings.by_widget.others.insert(k.clone(), config);
            };

            self.settings.monitors_v3.values_mut().for_each(|monitor| {
                if let Some(config) = monitor.by_widget.remove(&old_id) {
                    monitor.by_widget.insert(k.clone(), config);
                };
            });
            true
        });

        Ok(())
    }

    /// Sanitize wallpaper collections to remove non-existent wallpaper IDs
    pub(super) fn sanitize_wallpaper_collections(&mut self, resources: &ResourceManager) -> bool {
        let mut changed = false;
        let native = seelen_core::resource::WallpaperId::from(NATIVE_WALLPAPER_ID.to_owned());
        for collection in &mut self.settings.wallpaper_collections {
            let original_len = collection.wallpapers.len();
            collection
                .wallpapers
                // The virtual Windows wallpaper never depends on a file-backed entry.
                .retain(|wallpaper_id| {
                    wallpaper_id == &native || resources.wallpapers.contains_sync(wallpaper_id)
                });
            if collection.wallpapers.len() != original_len {
                changed = true;
            }
        }
        changed
    }

    pub fn write_settings(&self) -> Result<()> {
        self.settings.save(SEELEN_COMMON.settings_path())?;
        self.emit_settings()?;
        Ok(())
    }
}

fn show_corrupted_state_to_user(path: &Path) {
    let path = path.to_path_buf();
    std::thread::spawn(move || {
        let dialog = Dialog {
            title: vec![DialogContent::Group {
                items: vec![
                    DialogContent::Icon {
                        name: "BiSolidError".to_string(),
                        styles: Some(
                            CssStyles::new()
                                .add("color", "var(--color-red-800)")
                                .add("height", "1.2rem"),
                        ),
                    },
                    DialogContent::Text {
                        value: t!("runtime.corrupted_data").to_string(),
                        styles: None,
                    },
                ],
                styles: Some(CssStyles::new().add("alignItems", "center")),
            }],
            content: vec![
                DialogContent::Text {
                    value: t!("runtime.corrupted_file").to_string(),
                    styles: None,
                },
                DialogContent::Text {
                    value: format!("{}: {:?}", t!("runtime.corrupted_file_path"), path),
                    styles: None,
                },
            ],
            ..Default::default()
        };

        trigger_dialog_backend(dialog).log_error();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use seelen_core::{
        resource::WallpaperId,
        state::{Wallpaper, WallpaperCollection},
    };
    use std::sync::Arc;
    use uuid::Uuid;

    #[test]
    fn native_wallpaper_selection_survives_repeated_serialization_and_startup_cleanup() {
        let resources = ResourceManager::default(); // Built-ins may not be registered yet.
        let native = WallpaperId::from(NATIVE_WALLPAPER_ID.to_owned());
        let collection_id = Uuid::new_v4();
        let mut state = AppSettings {
            settings: Settings::default(),
        };
        state.settings.by_widget.wall.default_collection = Some(collection_id);
        state
            .settings
            .wallpaper_collections
            .push(WallpaperCollection {
                id: collection_id,
                name: "Current Windows wallpaper".into(),
                wallpapers: vec![native.clone()],
                hidden: true,
            });
        for _ in 0..3 {
            let saved = serde_json::to_string(&state.settings).unwrap();
            state.settings = serde_json::from_str(&saved).unwrap();
            assert!(!state.sanitize_wallpaper_collections(&resources));
            assert_eq!(
                state.settings.by_widget.wall.default_collection,
                Some(collection_id)
            );
            assert_eq!(state.settings.wallpaper_collections[0].id, collection_id);
            assert_eq!(
                state.settings.wallpaper_collections[0].wallpapers,
                vec![native.clone()]
            );
        }
    }

    #[test]
    fn native_wallpaper_protection_still_removes_missing_file_backed_wallpapers() {
        let resources = ResourceManager::default();
        let native = WallpaperId::from(NATIVE_WALLPAPER_ID.to_owned());
        let existing = WallpaperId::from("@test/existing".to_owned());
        let missing = WallpaperId::from("@test/missing".to_owned());
        resources
            .wallpapers
            .insert_sync(
                existing.clone(),
                Arc::new(Wallpaper {
                    id: existing.clone(),
                    ..Default::default()
                }),
            )
            .unwrap();
        let mut state = AppSettings {
            settings: Settings::default(),
        };
        state
            .settings
            .wallpaper_collections
            .push(WallpaperCollection {
                id: Uuid::new_v4(),
                name: "Mixed collection".into(),
                wallpapers: vec![native.clone(), existing.clone(), missing],
                hidden: false,
            });
        assert!(state.sanitize_wallpaper_collections(&resources));
        assert_eq!(
            state.settings.wallpaper_collections[0].wallpapers,
            vec![native, existing]
        );
        assert!(!state.sanitize_wallpaper_collections(&resources));
    }
}
