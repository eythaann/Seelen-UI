use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use seelen_core::{
    resource::{Resource, ResourceKind, SluResource, SluResourceFile},
    state::{CssStyles, IconPack, SluPopupConfig, SluPopupContent, Wallpaper},
};
use tauri::Listener;
use uuid::Uuid;

use crate::{
    error::Result,
    get_tokio_handle, log_error,
    state::application::{download_remote_icons, FULL_STATE},
    utils::{constants::SEELEN_COMMON, date_based_hex_id},
    widgets::popups::POPUPS_MANAGER,
};

pub const URI: &str = "seelen-ui.uri:";

pub fn process_uri(uri: &str) -> Result<()> {
    log::trace!("Loading URI: {uri}");

    if !uri.starts_with(URI) {
        let path = PathBuf::from(uri);
        if !path.is_file() || path.extension() != Some(OsStr::new("slu")) || !path.exists() {
            return Err("Invalid file to load".into());
        }

        let file = SluResourceFile::load(&path)?;
        store_file_on_respective_user_folder(&file)?;
        let id = POPUPS_MANAGER.lock().create(SluPopupConfig::default())?;
        update_popup_to_added_resource(&id, &file.resource)?;
        return Ok(());
    }

    let path = uri.trim_start_matches(URI).trim_start_matches("/");
    let parts = path.split("/").map(|s| s.to_string()).collect_vec();

    if parts.len() != 3 {
        return Err("Invalid URI format".into());
    }

    let [_method, enviroment, resource_id] = parts.as_slice() else {
        return Err("Invalid URI format".into());
    };

    let Ok(resource_id) = Uuid::parse_str(resource_id) else {
        return Err("Invalid URI format".into());
    };

    let env_prefix = if enviroment == "production" {
        "".to_string()
    } else {
        format!(".{enviroment}")
    };

    let url = format!("https://product{env_prefix}.seelen.io/resource/download/{resource_id}");
    get_tokio_handle().spawn(async move {
        log_error!(download_resource(&url).await);
    });

    Ok(())
}

fn path_by_resource_kind(kind: &ResourceKind) -> &Path {
    match kind {
        ResourceKind::Theme => SEELEN_COMMON.user_themes_path(),
        ResourceKind::IconPack => SEELEN_COMMON.user_icons_path(),
        ResourceKind::Widget => SEELEN_COMMON.user_widgets_path(),
        ResourceKind::Plugin => SEELEN_COMMON.user_plugins_path(),
        ResourceKind::Wallpaper => SEELEN_COMMON.user_wallpapers_path(),
        ResourceKind::SoundPack => SEELEN_COMMON.user_sounds_path(),
    }
}

fn store_file_on_respective_user_folder(file: &SluResourceFile) -> Result<PathBuf> {
    let mut path_to_store = path_by_resource_kind(&file.resource.kind).to_path_buf();
    if file.resource.kind == ResourceKind::IconPack || file.resource.kind == ResourceKind::Wallpaper
    {
        path_to_store.push(file.resource.id.to_string());
        std::fs::create_dir_all(&path_to_store)?;
        path_to_store.push("metadata.slu");
    } else {
        path_to_store.push(format!("{}.slu", file.resource.id));
    }
    file.store(&path_to_store)?;
    Ok(path_to_store)
}

async fn download_resource(url: &str) -> Result<()> {
    let popup_id = {
        POPUPS_MANAGER.lock().create(SluPopupConfig {
            title: vec![SluPopupContent::Group {
                items: vec![
                    SluPopupContent::Icon {
                        name: "TbCloudDownload".to_string(),
                        styles: Some(
                            CssStyles::new()
                                .add("color", "var(--color-blue-800)")
                                .add("height", "1.2rem"),
                        ),
                    },
                    SluPopupContent::Text {
                        value: t!("resource.downloading").to_string(),
                        styles: None,
                    },
                ],
                styles: Some(CssStyles::new().add("alignItems", "center")),
            }],
            content: vec![SluPopupContent::Text {
                value: t!("resource.downloading_body").to_string(),
                styles: None,
            }],
            ..Default::default()
        })?
    };

    let file = match _download_resource(url).await {
        Ok(file) => file,
        Err(err) => {
            POPUPS_MANAGER
                .lock()
                .update(&popup_id, error_popup_config(format!("{err:?}").as_str()))?;
            return Err(err);
        }
    };

    update_popup_to_added_resource(&popup_id, &file.resource)?;
    Ok(())
}

async fn _download_resource(url: &str) -> Result<SluResourceFile> {
    let res = reqwest::get(url).await?;
    let file = res.json::<SluResourceFile>().await?;
    let saved_path = store_file_on_respective_user_folder(&file)?;

    if file.resource.kind == ResourceKind::IconPack {
        let mut pack = IconPack::load(saved_path.parent().unwrap())?;
        download_remote_icons(&mut pack).await?;
    }

    if file.resource.kind == ResourceKind::Wallpaper {
        let mut wallpaper = Wallpaper::load(saved_path.parent().unwrap())?;
        download_remote_wallpapers(&mut wallpaper).await?;
    }

    Ok(file)
}

fn update_popup_to_added_resource(popup_id: &Uuid, resource: &Resource) -> Result<()> {
    let mut pupups_manager = POPUPS_MANAGER.lock();

    let config = resource_to_popup_config(resource)?;
    pupups_manager.update(popup_id, config)?;

    let id = resource.id;
    let friendly_id = resource.friendly_id.to_string();
    let kind = resource.kind.clone();
    let popup_id = *popup_id;

    let webview = pupups_manager
        .get_window_handle(&popup_id)
        .ok_or("Popup not found")?;
    let token = webview.once(format!("resource::{id}::enable"), move |_e| {
        std::thread::spawn(move || {
            FULL_STATE.rcu(move |state| {
                let mut state = state.cloned();
                match kind {
                    ResourceKind::Theme => {
                        state
                            .settings
                            .active_themes
                            .push(friendly_id.clone().into());
                    }
                    ResourceKind::IconPack => {
                        state
                            .settings
                            .active_icon_packs
                            .push(friendly_id.clone().into());
                    }
                    ResourceKind::Widget => {
                        state
                            .settings
                            .set_widget_enabled(&friendly_id.clone().into(), true);
                    }
                    ResourceKind::Wallpaper => {
                        state
                            .settings
                            .by_widget
                            .wall
                            .backgrounds_v2
                            .push(friendly_id.clone().into());
                    }
                    _ => {}
                }
                state
            });
            log_error!(FULL_STATE.load().write_settings());
            log_error!(POPUPS_MANAGER.lock().close_popup(&popup_id));
        });
    });

    pupups_manager
        .listeners
        .entry(resource.id)
        .or_default()
        .push(token);
    Ok(())
}

fn resource_to_popup_config(resource: &Resource) -> Result<SluPopupConfig> {
    let mut popup = SluPopupConfig {
        width: 480.0,
        height: 260.0,
        ..Default::default()
    };

    popup.title.push(SluPopupContent::Group {
        items: vec![
            SluPopupContent::Icon {
                name: "GrCircleInformation".to_string(),
                styles: Some(
                    CssStyles::new()
                        .add("color", "var(--color-blue-900)")
                        .add("height", "1.2rem"),
                ),
            },
            SluPopupContent::Text {
                value: t!("resource.added").to_string(),
                styles: None,
            },
        ],
        styles: Some(CssStyles::new().add("alignItems", "center")),
    });

    let image_styles = CssStyles::new()
        .add("width", "90px")
        .add("minWidth", "90px")
        .add("height", "90px")
        .add("borderRadius", "14px")
        .add("backgroundColor", "var(--color-gray-200)")
        .add("display", "flex")
        .add("alignItems", "center")
        .add("justifyContent", "center");

    let image = if let Some(url) = &resource.metadata.portrait {
        SluPopupContent::Image {
            href: url.clone(),
            styles: Some(image_styles),
        }
    } else {
        SluPopupContent::Icon {
            name: "GrStatusUnknown".to_string(),
            styles: Some(image_styles),
        }
    };

    let state = FULL_STATE.load();
    let locale = state.locale();
    popup.content = vec![SluPopupContent::Group {
        items: vec![
            image,
            SluPopupContent::Group {
                items: vec![
                    SluPopupContent::Text {
                        value: resource.metadata.display_name.get(locale).to_owned(),
                        styles: Some(
                            CssStyles::new()
                                .add("fontWeight", "bold")
                                .add("fontSize", "2rem")
                                .add("lineHeight", "1.2em"),
                        ),
                    },
                    SluPopupContent::Text {
                        value: resource.metadata.description.get(locale).to_owned(),
                        styles: None,
                    },
                ],
                styles: Some(CssStyles::new().add("flexDirection", "column")),
            },
        ],
        styles: None,
    }];

    popup.footer = vec![SluPopupContent::Button {
        inner: vec![SluPopupContent::Text {
            value: t!("resource.enable").to_string(),
            styles: None,
        }],
        on_click: format!("resource::{}::enable", resource.id),
        styles: None,
    }];

    Ok(popup)
}

fn error_popup_config(err: &str) -> SluPopupConfig {
    SluPopupConfig {
        title: vec![SluPopupContent::Group {
            items: vec![
                SluPopupContent::Icon {
                    name: "BiSolidError".to_string(),
                    styles: Some(
                        CssStyles::new()
                            .add("color", "var(--color-red-800)")
                            .add("height", "1.2rem"),
                    ),
                },
                SluPopupContent::Text {
                    value: t!("resource.download_failed_title").to_string(),
                    styles: None,
                },
            ],
            styles: Some(CssStyles::new().add("alignItems", "center")),
        }],
        content: vec![
            SluPopupContent::Text {
                value: t!("resource.download_failed_body").to_string(),
                styles: None,
            },
            SluPopupContent::Text {
                value: "=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=".to_string(),
                styles: Some(CssStyles::new().add("color", "var(--color-gray-400)")),
            },
            SluPopupContent::Text {
                value: format!("Error: {err}"),
                styles: None,
            },
        ],
        ..Default::default()
    }
}

async fn download_remote_wallpapers(wallpaper: &mut Wallpaper) -> Result<()> {
    if wallpaper.url.is_none() && wallpaper.thumbnail_url.is_none() {
        return Ok(());
    }

    let folder_to_store = &wallpaper.metadata.internal.path;

    // Download the main wallpaper file
    if let Some(url) = &wallpaper.url {
        let filename = download_remote_asset(url, folder_to_store).await?;
        wallpaper.filename = Some(filename);
    }

    // Download the thumbnail
    if let Some(thumbnail_url) = &wallpaper.thumbnail_url {
        let thumbnail_filename = download_remote_asset(thumbnail_url, folder_to_store).await?;
        wallpaper.thumbnail_filename = Some(thumbnail_filename);
    }

    wallpaper.save()?;
    Ok(())
}

async fn download_remote_asset(url: &url::Url, folder_to_store: &Path) -> Result<String> {
    if !folder_to_store.is_dir() {
        std::fs::create_dir_all(folder_to_store)?;
    }

    let Some(extension) = url.path().split('.').next_back() else {
        return Err("Could not determine file extension from URL".into());
    };

    let res = reqwest::get(url.as_str()).await?;
    let bytes = res.bytes().await?;

    let filename = format!("{}.{}", date_based_hex_id(), extension);
    let file_path = folder_to_store.join(&filename);

    std::fs::write(&file_path, &bytes)?;
    Ok(filename)
}
