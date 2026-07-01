use std::path::Path;

use slu_utils::checksums::CheckSums;
use walkdir::WalkDir;

use crate::error::Result;

use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use super::IntegrityError;

/// Public key for minisign verification (same as updater)
const MINISIGN_PUBLIC_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDQ4QjU1RUI0NEM0NzBERUIKUldUckRVZE10RjYxU0lpaERvdklYL05DVlg0Sk9EVngvaEgzZjMvU1NNemJTZXZ1K0dNVXU3ZkQK";

pub async fn ensure_bundle_files_integrity(
    app: &tauri::AppHandle,
) -> std::result::Result<(), IntegrityError> {
    if let Err(e) = try_ensure_bundle_files_integrity(app).await {
        log::error!("Bundle integrity check failed: {e:?}");
        return Err(IntegrityError::BundleIntegrityFailed);
    }
    Ok(())
}

async fn try_ensure_bundle_files_integrity(app: &tauri::AppHandle) -> Result<()> {
    log::trace!("Validating bundle files integrity");

    let install_dir = app.path().resource_dir()?;
    let static_path = install_dir.join("static");

    let checksums_path = install_dir.join("SHA256SUMS");
    let signature_path = install_dir.join("SHA256SUMS.sig");

    if !signature_path.exists() {
        return Err("Signature file not found".into());
    }

    if !checksums_path.exists() {
        return Err("Checksums file not found".into());
    }

    // Skip signature validation in development mode
    if !tauri::is_dev() {
        verify_external_signature(&checksums_path, &signature_path, MINISIGN_PUBLIC_KEY)?;
    }
    validate_directory_checksums(&static_path, &checksums_path).await?;

    Ok(())
}

async fn validate_directory_checksums(base_path: &Path, checksums_path: &Path) -> Result<()> {
    log::trace!("Validating checksums for {}", base_path.display());

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

        // Read file using full path, but store checksum with relative path
        let content =
            std::fs::read(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        actual_checksums.raw_add(&content, relative_path);
    }

    let diffs = expected_checksums.compare(&actual_checksums);
    if !diffs.is_empty() {
        log::error!("Checksums mismatch: {:#?}", diffs);
        return Err("Checksums does not match".into());
    }

    log::trace!("All Checksums validated successfully");
    Ok(())
}

fn verify_external_signature(file: &Path, signature_file: &Path, key_base64: &str) -> Result<()> {
    let checksums_content = std::fs::read(file)?;
    let signature_content = std::fs::read_to_string(signature_file)?;

    // Unsigned builds (forks / self-built releases without the signing secret)
    // ship a placeholder in place of a real minisign signature. Skip the
    // cryptographic check for those; file integrity is still enforced by the
    // checksum comparison in validate_directory_checksums().
    if signature_content.trim() == slu_utils::signature::UNSIGNED_MARKER {
        log::warn!("Bundle is UNSIGNED; skipping signature verification (checksums still enforced).");
        return Ok(());
    }

    slu_utils::signature::verify_minisign(&checksums_content, &signature_content, key_base64)?;
    log::trace!("Signature verification successful for {}", file.display());
    Ok(())
}

pub fn show_bundle_integrity_dialog(app: &tauri::AppHandle) {
    app.dialog()
        .message(t!("runtime.files_integrity"))
        .title(t!("runtime.files_integrity_title"))
        .kind(MessageDialogKind::Error)
        .blocking_show();
}
