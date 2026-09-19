use std::path::Path;

use futures::StreamExt;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use slu_utils::checksums::{CheckSums, calculate_sha256};

use crate::error::Result;

use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

/// Public key for minisign verification (same as updater)
const MINISIGN_PUBLIC_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDQ4QjU1RUI0NEM0NzBERUIKUldUckRVZE10RjYxU0lpaERvdklYL05DVlg0Sk9EVngvaEgzZjMvU1NNemJTZXZ1K0dNVXU3ZkQK";

pub async fn ensure_bundle_files_integrity(app: &tauri::AppHandle) -> Result<()> {
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

    let checksums_content = tokio::fs::read(checksums_path).await?;
    let expected_checksums = CheckSums::parse(&checksums_content)?;

    let root = base_path.parent().unwrap();
    let walk_path = base_path.to_path_buf();
    let files =
        tokio::task::spawn_blocking(move || crate::utils::collect_files(walk_path.as_path()))
            .await
            .map_err(|e| format!("Bundle files listing task failed: {e}"))?;

    let contents = futures::stream::iter(files)
        .map(|path| async move {
            let content = tokio::fs::read(&path).await.ok()?;
            Some((path, content))
        })
        .buffer_unordered(64)
        .filter_map(futures::future::ready)
        .collect::<Vec<_>>()
        .await;

    // Hashing is CPU bound, so it is spread between the cores off the runtime workers
    let hashes = tokio::task::spawn_blocking(move || {
        contents
            .into_par_iter()
            .map(|(path, content)| (path, calculate_sha256(&content)))
            .collect::<Vec<_>>()
    })
    .await
    .map_err(|e| format!("Bundle files hashing task failed: {e}"))?;

    // Store checksums with the path relative to the bundle root
    let mut actual_checksums = CheckSums::new();
    for (path, hash) in hashes {
        let relative_path = path.strip_prefix(root).expect("Strip failed");
        actual_checksums.add_hash(hash, relative_path);
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
