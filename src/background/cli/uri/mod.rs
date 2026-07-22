mod icons_downloader;

use std::{
    ffi::OsStr,
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

use itertools::Itertools;
use seelen_core::{
    resource::{Resource, ResourceId, ResourceKind, SluResource, SluResourceFile},
    state::{CssStyles, Dialog, DialogContent, IconPack, Wallpaper},
};
use tauri::Listener;
use uuid::Uuid;

use crate::{
    app::get_app_handle,
    cli::uri::icons_downloader::download_remote_icons,
    error::Result,
    error::ResultLogExt,
    get_tokio_handle,
    resources::RESOURCES,
    session::application::SessionManager,
    state::application::FULL_STATE,
    utils::{constants::SEELEN_COMMON, date_based_hex_id},
    widgets::{show_settings_at, trigger_dialog_backend},
};

pub const URI: &str = "seelen-ui.uri:";

pub async fn process_uri(uri: &str) -> Result<()> {
    log::trace!("Loading URI: {uri}");

    if !uri.starts_with(URI) {
        let path = PathBuf::from(uri);
        if !path.is_file() || path.extension() != Some(OsStr::new("slu")) || !path.exists() {
            return Err("Invalid file to load".into());
        }

        let file = SluResourceFile::load(&path).await?;
        store_file_on_respective_user_folder(&file).await?;

        let dialog_id = Uuid::new_v4();
        trigger_dialog_backend(Dialog {
            identifier: dialog_id,
            ..Default::default()
        })?;
        update_dialog_to_added_resource(dialog_id, &file.resource)?;
        return Ok(());
    }

    let path = uri.trim_start_matches(URI).trim_start_matches("/");

    // auth/callback?code=<token>&state=<state>
    if path.starts_with("auth/callback") {
        let query = path.split_once('?').map(|x| x.1).unwrap_or("");
        let params: std::collections::HashMap<_, _> = url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();
        let code = params
            .get("code")
            .ok_or("Auth callback missing 'code' parameter")?
            .clone();
        let state = params
            .get("state")
            .ok_or("Auth callback missing 'state' parameter")?
            .clone();

        get_tokio_handle().spawn(async move {
            if let Err(err) = SessionManager::handle_auth_callback(code, state).await {
                log::error!("Auth callback failed: {err:?}");
                trigger_dialog_backend(auth_error_dialog(&format!("{err:?}"))).log_error();
            }
        });
        return Ok(());
    }

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
    download_resource(&url).await?;
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

async fn store_file_on_respective_user_folder(file: &SluResourceFile) -> Result<PathBuf> {
    let mut path_to_store = path_by_resource_kind(&file.resource.kind).to_path_buf();
    if file.resource.kind == ResourceKind::IconPack || file.resource.kind == ResourceKind::Wallpaper
    {
        path_to_store.push(file.resource.id.to_string());
        tokio::fs::create_dir_all(&path_to_store).await?;
        path_to_store.push("metadata.slu");
    } else {
        path_to_store.push(format!("{}.slu", file.resource.id));
    }
    file.store(&path_to_store).await?;
    Ok(path_to_store)
}

async fn download_resource(url: &str) -> Result<()> {
    let dialog_id = Uuid::new_v4();

    trigger_dialog_backend(Dialog {
        identifier: dialog_id,
        title: vec![DialogContent::Group {
            items: vec![
                DialogContent::Icon {
                    name: "TbCloudDownload".to_string(),
                    styles: Some(
                        CssStyles::new()
                            .add("color", "var(--color-blue-800)")
                            .add("height", "1.2rem"),
                    ),
                },
                DialogContent::Text {
                    value: t!("resource.downloading").to_string(),
                    styles: None,
                },
            ],
            styles: Some(CssStyles::new().add("alignItems", "center")),
        }],
        content: vec![
            DialogContent::Text {
                value: t!("resource.downloading_body").to_string(),
                styles: None,
            },
            DialogContent::Loader {
                styles: Some(CssStyles::new().add("marginTop", "12px")),
            },
        ],
        ..Default::default()
    })?;

    let file = match _download_resource(url).await {
        Ok(file) => file,
        Err(err) if err.code() == reqwest::StatusCode::UNAUTHORIZED.as_u16() => {
            let event = "open_settings_extras";
            get_app_handle().once(event, move |_| {
                show_settings_at("/extras").log_error();
            });
            trigger_dialog_backend(login_required_dialog(dialog_id, event)).log_error();
            return Err(err);
        }
        Err(err) => {
            trigger_dialog_backend(error_dialog(dialog_id, format!("{err:?}").as_str()))
                .log_error();
            return Err(err);
        }
    };

    update_dialog_to_added_resource(dialog_id, &file.resource)?;
    Ok(())
}

async fn _download_resource(url: &str) -> Result<SluResourceFile> {
    let res = SessionManager::authed_get(url).send().await?;
    let status = res.status();
    if !status.is_success() {
        let body = res.text().await.unwrap_or_default();
        log::error!("Failed to download resource: {status} - {body}");
        return Err(status.into());
    }

    let file = res.json::<SluResourceFile>().await?;
    let saved_path = store_file_on_respective_user_folder(&file).await?;

    if file.resource.kind == ResourceKind::IconPack {
        let mut pack = IconPack::load(saved_path.parent().unwrap()).await?;
        download_remote_icons(&mut pack).await?;
    }

    if file.resource.kind == ResourceKind::Wallpaper {
        let mut wallpaper = Wallpaper::load(saved_path.parent().unwrap()).await?;
        download_remote_wallpapers(&mut wallpaper).await?;
    }

    Ok(file)
}

fn update_dialog_to_added_resource(dialog_id: Uuid, resource: &Resource) -> Result<()> {
    let config = resource_to_dialog(dialog_id, resource)?;
    trigger_dialog_backend(config)?;

    let used_id = ResourceId::Remote(resource.id);
    let kind = resource.kind;

    let event = format!("resource::{}::enable", resource.id);
    get_app_handle().once(event, move |_| {
        RESOURCES.enable_resource(kind, used_id.clone());
    });

    Ok(())
}

fn resource_to_dialog(id: Uuid, resource: &Resource) -> Result<Dialog> {
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
        DialogContent::Image {
            href: url.clone(),
            styles: Some(image_styles),
        }
    } else {
        DialogContent::Icon {
            name: "GrStatusUnknown".to_string(),
            styles: Some(image_styles),
        }
    };

    let state = FULL_STATE.load();
    let locale = state.locale();

    Ok(Dialog {
        identifier: id,
        width: 500.0,
        height: 280.0,
        title: vec![DialogContent::Group {
            items: vec![
                DialogContent::Icon {
                    name: "GrCircleInformation".to_string(),
                    styles: Some(
                        CssStyles::new()
                            .add("color", "var(--color-blue-900)")
                            .add("height", "1.2rem"),
                    ),
                },
                DialogContent::Text {
                    value: t!("resource.added").to_string(),
                    styles: None,
                },
            ],
            styles: Some(CssStyles::new().add("alignItems", "center")),
        }],
        content: vec![DialogContent::Group {
            items: vec![
                image,
                DialogContent::Group {
                    items: vec![
                        DialogContent::Text {
                            value: resource.metadata.display_name.get(locale).to_owned(),
                            styles: Some(
                                CssStyles::new()
                                    .add("fontWeight", "bold")
                                    .add("fontSize", "2rem")
                                    .add("lineHeight", "1.2em"),
                            ),
                        },
                        DialogContent::Text {
                            value: resource.metadata.description.get(locale).to_owned(),
                            styles: None,
                        },
                    ],
                    styles: Some(CssStyles::new().add("flexDirection", "column")),
                },
            ],
            styles: None,
        }],
        footer: vec![DialogContent::Button {
            skin: Some("solid".to_string()),
            inner: vec![DialogContent::Text {
                value: t!("resource.enable").to_string(),
                styles: None,
            }],
            on_click: format!("resource::{}::enable", resource.id),
            styles: None,
        }],
    })
}

fn auth_error_dialog(err: &str) -> Dialog {
    Dialog {
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
                    value: t!("auth.login_failed_title").to_string(),
                    styles: None,
                },
            ],
            styles: Some(CssStyles::new().add("alignItems", "center")),
        }],
        content: vec![
            DialogContent::Text {
                value: t!("auth.login_failed_body").to_string(),
                styles: None,
            },
            DialogContent::Text {
                value: "=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=".to_string(),
                styles: Some(CssStyles::new().add("color", "var(--color-gray-400)")),
            },
            DialogContent::Text {
                value: format!("Error: {err}"),
                styles: None,
            },
        ],
        ..Default::default()
    }
}

fn login_required_dialog(id: Uuid, login_event: &str) -> Dialog {
    Dialog {
        identifier: id,
        title: vec![DialogContent::Group {
            items: vec![
                DialogContent::Icon {
                    name: "LuUserRound".to_string(),
                    styles: Some(
                        CssStyles::new()
                            .add("color", "var(--color-blue-800)")
                            .add("height", "1.2rem"),
                    ),
                },
                DialogContent::Text {
                    value: t!("resource.login_required_title").to_string(),
                    styles: None,
                },
            ],
            styles: Some(CssStyles::new().add("alignItems", "center")),
        }],
        content: vec![DialogContent::Text {
            value: t!("resource.login_required_body").to_string(),
            styles: None,
        }],
        footer: vec![DialogContent::Button {
            skin: Some("solid".to_string()),
            inner: vec![DialogContent::Text {
                value: t!("resource.login_required_action").to_string(),
                styles: None,
            }],
            on_click: login_event.to_string(),
            styles: None,
        }],
        ..Default::default()
    }
}

fn error_dialog(id: Uuid, err: &str) -> Dialog {
    Dialog {
        identifier: id,
        title: vec![DialogContent::Group {
            items: vec![
                DialogContent::Icon {
                    name: "BiSolidError".to_string(),
                    styles: Some(
                        CssStyles::new()
                            .add("color", "var(--color-red-600)")
                            .add("height", "1.2rem"),
                    ),
                },
                DialogContent::Text {
                    value: t!("resource.download_failed_title").to_string(),
                    styles: None,
                },
            ],
            styles: Some(CssStyles::new().add("alignItems", "center")),
        }],
        content: vec![
            DialogContent::Text {
                value: t!("resource.download_failed_body").to_string(),
                styles: None,
            },
            DialogContent::Text {
                value: "=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=".to_string(),
                styles: Some(CssStyles::new().add("color", "var(--color-gray-400)")),
            },
            DialogContent::Text {
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

    let folder_to_store = wallpaper.metadata.directory()?;

    // Download the main wallpaper file
    if let Some(url) = &wallpaper.url {
        let filename = download_remote_asset(url, &folder_to_store).await?;
        wallpaper.filename = Some(filename);
    }

    // Download the thumbnail
    if let Some(thumbnail_url) = &wallpaper.thumbnail_url {
        let thumbnail_filename = download_remote_asset(thumbnail_url, &folder_to_store).await?;
        wallpaper.thumbnail_filename = Some(thumbnail_filename);
    }

    wallpaper.save().await?;
    Ok(())
}

async fn download_remote_asset(url: &url::Url, folder_to_store: &Path) -> Result<String> {
    if !folder_to_store.is_dir() {
        std::fs::create_dir_all(folder_to_store)?;
    }

    let Some(extension) = url.path().split('.').next_back() else {
        return Err("Could not determine file extension from URL".into());
    };

    // Use a long timeout for large assets (videos can be hundreds of MB; the
    // global 15 s client timeout is too short on slow / CDN-throttled connections).
    let res = SessionManager::plain_get(url.as_str())
        .timeout(Duration::from_secs(3600))
        .send()
        .await?;
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Failed to download asset: {status} - {body}").into());
    }

    let filename = format!("{}.{}", date_based_hex_id(), extension);
    let file_path = folder_to_store.join(&filename);

    // Stream chunks to disk to avoid buffering the whole file in memory.
    let mut file = std::fs::File::create(&file_path)?;
    let mut stream = res;
    while let Some(chunk) = stream.chunk().await? {
        file.write_all(&chunk)?;
    }
    file.flush()?;

    Ok(filename)
}
