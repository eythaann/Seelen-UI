use std::path::Path;

use slu_utils::checksums::CheckSums;
use walkdir::WalkDir;

use crate::{app::get_app_handle, error::Result};

use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

/// Public key for minisign verification (same as updater)
const MINISIGN_PUBLIC_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDQ4QjU1RUI0NEM0NzBERUIKUldUckRVZE10RjYxU0lpaERvdklYL05DVlg0Sk9EVngvaEgzZjMvU1NNemJTZXZ1K0dNVXU3ZkQK";

pub fn ensure_bundle_files_integrity(app: &tauri::AppHandle) -> Result<()> {
    log::trace!("Validating bundle files integrity");

    let install_dir = app.path().resource_dir()?;
    let static_path = install_dir.join("static");

    let checksums_path = install_dir.join("SHA256SUMS");
    let signature_path = install_dir.join("SHA256SUMS.sig");

    if !signature_path.exists() {
        show_integrity_dialog();
        return Err("Signature file not found".into());
    }

    if !checksums_path.exists() {
        show_integrity_dialog();
        return Err("Checksums file not found".into());
    }

    // Skip signature validation in development mode
    if !tauri::is_dev() {
        if let Err(err) =
            verify_external_signature(&checksums_path, &signature_path, MINISIGN_PUBLIC_KEY)
        {
            show_integrity_dialog();
            return Err(err);
        }
    }

    if let Err(err) = validate_directory_checksums(&static_path, &checksums_path) {
        show_integrity_dialog();
        return Err(err);
    }

    Ok(())
}

fn validate_directory_checksums(base_path: &Path, checksums_path: &Path) -> Result<()> {
    log::info!("Validating checksums for {}", base_path.display());

    let checksums_content = std::fs::read(checksums_path)?;
    let expected_checksums = CheckSums::parse(&checksums_content)?;

    // Calculate actual checksums
    let mut actual_checksums = CheckSums::new();
    for entry in WalkDir::new(base_path)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let relative_path = path
            .strip_prefix(base_path.parent().unwrap())
            .expect("Strip failed");
        actual_checksums.add(relative_path)?;
    }

    let diffs = expected_checksums.compare(&actual_checksums);
    if !diffs.is_empty() {
        log::error!("Checksums mismatch: {:#?}", diffs);
        return Err("Checksums does not match".into());
    }

    log::info!("All Checksums validated successfully");
    Ok(())
}

fn verify_external_signature(file: &Path, signature_file: &Path, key_base64: &str) -> Result<()> {
    let checksums_content = std::fs::read(file)?;
    let signature_content = std::fs::read_to_string(signature_file)?;

    slu_utils::signature::verify_minisign(&checksums_content, &signature_content, key_base64)?;
    log::info!("Signature verification successful for {}", file.display());
    Ok(())
}

/// Shows an error dialog for integrity validation failures
fn show_integrity_dialog() {
    get_app_handle()
        .dialog()
        .message(t!("runtime.files_integrity"))
        .title(t!("runtime.files_integrity_title"))
        .kind(MessageDialogKind::Error)
        .blocking_show();
}
