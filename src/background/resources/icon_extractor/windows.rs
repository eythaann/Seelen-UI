use std::{
    io::BufRead,
    path::{Path, PathBuf},
};

use image::RgbaImage;
use seelen_core::state::Icon;
use windows::Win32::{
    Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
    UI::{
        Controls::{IImageList, ILD_TRANSPARENT},
        Shell::{
            ExtractIconExW, SHFILEINFOW, SHGFI_SYSICONINDEX, SHGetFileInfoW, SHGetImageList,
            SHIL_JUMBO,
        },
        WindowsAndMessaging::HICON,
    },
};
use windows_core::Owned;

use crate::{
    error::Result,
    resources::RESOURCES,
    system_modules::start::StartMenuManager,
    utils::{constants::SEELEN_COMMON, date_based_hex_id},
    windows_api::{
        WindowsApi, icon::convert_hicon_to_rgba_image, msix::MsixAppsManager,
        string_utils::WindowsString, types::AppUserModelId,
    },
};

use super::{crop_transparent_borders, is_aproximately_a_square, save_as_webp};

/// Failures of these apps can be fixed by a start menu refresh (their shortcut was not found yet).
pub fn depends_on_start_menu(app_id: &str) -> bool {
    AppUserModelId::from(app_id.to_owned()).is_property_store()
}

fn extract_icon_from_module(path: &Path, index: i32) -> Result<RgbaImage> {
    let path = WindowsString::from(path);
    unsafe {
        let mut hicon = HICON::default();
        let extracted = ExtractIconExW(path.as_pcwstr(), index, Some(&mut hicon), None, 1);
        if extracted == 0 || hicon.is_invalid() {
            return Err(format!("Icon index {index} not found").into());
        }
        let hicon = Owned::new(hicon);

        let image = crop_transparent_borders(&convert_hicon_to_rgba_image(&hicon)?);
        Ok(image)
    }
}

fn get_shell_icon(path: &Path) -> Result<RgbaImage> {
    unsafe {
        let normalized = path
            .canonicalize()?
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .to_owned();
        let path = WindowsString::from(normalized);

        let mut file_info = SHFILEINFOW::default();
        let result = SHGetFileInfoW(
            path.as_pcwstr(),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut file_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_SYSICONINDEX,
        );

        if result == 0 {
            return Err("Failed to get file information".into());
        }

        // file_info.iIcon = 0 is a valid icon but it is the default icon for files on Windows
        // so we will handle this as no icon to avoid generate unnecessary artifacts
        if file_info.iIcon == 0 {
            return Err("Icon index is 0".into());
        }

        let image_list: IImageList = SHGetImageList(SHIL_JUMBO as i32)?;
        // if 256x256 icon is not available, will use the icons with the most color depth and size
        // this is useful for some icons where color depth is less than 32,
        // example: icon of 124x124 16bits and other 64x64 32bits this will return the 32bits icon
        // color depth is prioritized over size
        let icon = Owned::new(image_list.GetIcon(file_info.iIcon, ILD_TRANSPARENT.0)?);
        let image = crop_transparent_borders(&convert_hicon_to_rgba_image(&icon)?);
        Ok(image)
    }
}

fn get_icon_from_url_file(path: &Path) -> Result<RgbaImage> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);

    let mut icon_file = None;
    let mut icon_idx: Option<i32> = None;

    // in theory .url files are encoded in UTF-8 so we don't need to use OsString
    for line in reader.lines() {
        let line = line?;
        if let Some(stripped) = line.strip_prefix("IconFile=") {
            icon_file = Some(PathBuf::from(stripped));
        } else if let Some(stripped) = line.strip_prefix("IconIndex=") {
            icon_idx = Some(stripped.parse()?);
        }
    }

    let icon_file = icon_file.ok_or("Url does not have IconFile")?;
    extract_icon_from_module(&icon_file, icon_idx.unwrap_or(0))
}

/// Parses Windows-style `path,index` icon notation (e.g. `C:\foo\bar.ico,0`).
///
/// Returns `(path, index)` only when the string ends with `,<integer>` **and** the path
/// before the comma points to an existing file.
fn parse_path_with_icon_index(raw: &Path) -> Option<(PathBuf, i32)> {
    let s = raw.to_str()?;
    let comma = s.rfind(',')?;
    let index: i32 = s[comma + 1..].trim().parse().ok()?;
    let path = PathBuf::from(s[..comma].trim());
    if path.exists() && !path.is_dir() {
        Some((path, index))
    } else {
        None
    }
}

/// Extracts an icon from a file at a specific resource index and saves it.
///
/// `key` is the original `path,index` string (e.g. `C:\foo\bar.ico,0`) used as the lookup
/// key in RESOURCES, so different indices of the same file are stored independently.
/// `path` is the actual file path without the index suffix.
fn extract_from_module_with_index(key: &Path, path: &Path, index: i32) -> Result<()> {
    if RESOURCES.has_app_icon(None, Some(key)) {
        return Ok(());
    }

    let filestem = path.file_stem().ok_or("Failed to get file stem")?;
    let gen_icon_filename = format!(
        "{}_{}.webp",
        filestem.to_string_lossy(),
        date_based_hex_id()
    );

    log::trace!("Extracting icon (index {index}) for {:?}", path.file_name());

    let image = extract_icon_from_module(path, index)?;
    let image = crop_transparent_borders(&image);

    let gen_icon = Icon {
        base: Some(gen_icon_filename.clone()),
        is_aproximately_square: is_aproximately_a_square(&image),
        ..Default::default()
    };

    save_as_webp(
        &image,
        &SEELEN_COMMON
            .system_icon_pack_path()
            .join(&gen_icon_filename),
    )?;
    RESOURCES.add_system_app_icon(None, Some(key), gen_icon);
    Ok(())
}

/// returns the path of the icon extracted from the executable or copied if is an UWP app.
///
/// If the icon already exists, it returns the path instead overriding, this is needed for allow user custom icons.
///
/// umid on this case only applys to Property Store umid
pub fn extract_from_file(origin: &Path) -> Result<()> {
    if !origin.exists() || origin.is_dir() {
        // Handle Windows path,index notation (e.g. "C:\path\app.exe,2" or "file.ico,0")
        if let Some((real_path, index)) = parse_path_with_icon_index(origin) {
            return extract_from_module_with_index(origin, &real_path, index);
        }
        return Err(format!("File not found: {}", origin.display()).into());
    }

    let origin_ext = match origin.extension() {
        Some(ext) => ext.to_string_lossy().to_lowercase(),
        // no extension === no icon
        None => return Ok(()),
    };

    // ico files are by itself an icon
    if origin_ext == "ico" {
        return Ok(());
    }

    let is_exe_file = origin_ext == "exe";
    let is_lnk_file = origin_ext == "lnk";
    let is_url_file = origin_ext == "url";

    if is_exe_file || is_lnk_file || is_url_file {
        if RESOURCES.has_app_icon(None, Some(origin)) {
            return Ok(());
        }
    } else if RESOURCES.has_shared_file_icon(origin) {
        return Ok(());
    }

    let file_name = origin.file_name().ok_or("Failed to get file name")?;
    let filestem = origin.file_stem().ok_or("Failed to get file stem")?;

    let gen_icon_filename = format!(
        "{}_{}.webp",
        filestem.to_string_lossy(),
        date_based_hex_id()
    );
    let mut gen_icon = Icon {
        base: Some(gen_icon_filename.clone()),
        ..Default::default()
    };

    log::trace!("Extracting icon for {file_name:?}");

    if origin_ext == "url" {
        let image = get_icon_from_url_file(origin)?;
        gen_icon.is_aproximately_square = is_aproximately_a_square(&image);
        save_as_webp(
            &image,
            &SEELEN_COMMON
                .system_icon_pack_path()
                .join(&gen_icon_filename),
        )?;
        RESOURCES.add_system_app_icon(None, Some(origin), gen_icon);
        return Ok(());
    }

    let mut umid = None;
    if is_lnk_file {
        umid = WindowsApi::get_file_umid(origin).ok();
        let has_custom_icon = WindowsApi::resolve_lnk_custom_icon_path(origin).is_ok();

        if !has_custom_icon {
            let (target, ..) = WindowsApi::resolve_lnk_target(origin)?;
            extract_from_file(&target)?;
            RESOURCES.add_system_icon_redirect(umid, origin, &target);
            return Ok(());
        }
    }

    // try get the icon directly from the file
    let icon = get_shell_icon(origin)?;
    gen_icon.is_aproximately_square = is_aproximately_a_square(&icon);

    if is_exe_file || is_lnk_file {
        save_as_webp(
            &icon,
            &SEELEN_COMMON
                .system_icon_pack_path()
                .join(&gen_icon_filename),
        )?;
        RESOURCES.add_system_app_icon(umid.as_deref(), Some(origin), gen_icon);
    } else {
        let gen_icon_filename = format!("{}_{}.webp", origin_ext, date_based_hex_id());
        save_as_webp(
            &icon,
            &SEELEN_COMMON
                .system_icon_pack_path()
                .join(&gen_icon_filename),
        )?;
        gen_icon.base = Some(gen_icon_filename);
        RESOURCES.add_system_file_icon(&origin_ext, gen_icon);
    }

    Ok(())
}

/// Extracts and saves the icon of the app with the specified app user model id.
pub fn extract_from_app_id(app_id: &str) -> Result<()> {
    let aumid = AppUserModelId::from(app_id.to_owned());
    match &aumid {
        AppUserModelId::Appx(app_umid) => {
            let msix_manager = MsixAppsManager::instance();
            let path = msix_manager.get_app_path(app_umid)?;
            {
                if RESOURCES.has_app_icon(Some(aumid.as_str()), path.as_deref()) {
                    return Ok(());
                }
            }

            log::trace!("Extracting icon for {app_umid:?}");
            let mut gen_icon = Icon::default();
            let (light_path, dark_path) = msix_manager.get_app_icon_path(app_umid)?;

            let name = date_based_hex_id();

            let light_rgba = image::open(&light_path)?.to_rgba8();
            let light_rgba = crop_transparent_borders(&light_rgba);

            if light_path != dark_path {
                let dark_rgba = image::open(&dark_path)?.to_rgba8();
                let dark_rgba = crop_transparent_borders(&dark_rgba);

                save_as_webp(
                    &light_rgba,
                    &SEELEN_COMMON
                        .system_icon_pack_path()
                        .join(format!("{name}_light.webp")),
                )?;
                save_as_webp(
                    &dark_rgba,
                    &SEELEN_COMMON
                        .system_icon_pack_path()
                        .join(format!("{name}_dark.webp")),
                )?;

                gen_icon.light = Some(format!("{name}_light.webp"));
                gen_icon.dark = Some(format!("{name}_dark.webp"));
            } else {
                save_as_webp(
                    &light_rgba,
                    &SEELEN_COMMON
                        .system_icon_pack_path()
                        .join(format!("{name}.webp")),
                )?;
                gen_icon.base = Some(format!("{name}.webp"));
            }

            gen_icon.is_aproximately_square = is_aproximately_a_square(&light_rgba);

            RESOURCES.add_system_app_icon(Some(app_umid), path.as_deref(), gen_icon);
            Ok(())
        }
        AppUserModelId::PropertyStore(app_umid) => {
            let start = StartMenuManager::instance();
            let lnk = start
                .get_by_file_umid(app_umid)
                .ok_or(format!("No shortcut found for umid {app_umid}"))?;

            {
                if RESOURCES.has_app_icon(Some(aumid.as_str()), Some(&lnk.path)) {
                    return Ok(());
                }
            }

            extract_from_file(&lnk.path)?;
            Ok(())
        }
    }
}
