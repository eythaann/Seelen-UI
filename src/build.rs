use std::{fs::create_dir, path::PathBuf};

use slu_utils::{checksums::CheckSums, signature::sign_minisign};

fn main() {
    let _ = create_dir("gen");

    let mut checksums = CheckSums::new();
    read_folder_recursive(PathBuf::from("static"), &mut |path| {
        checksums.add(&path).unwrap();
    });

    let target_dir = target_dir();
    let sums_path = target_dir.join("SHA256SUMS");
    checksums.write(&sums_path).unwrap();

    if !cfg!(debug_assertions) {
        sign_sha256sums(&sums_path);
    } else {
        std::fs::write(
            sums_path.with_extension("sig"),
            "NOT SIGNED NEEDED FOR DEBUG",
        )
        .unwrap();
    }

    tauri_build::build();
}

fn target_dir() -> PathBuf {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    // see <https://github.com/rust-lang/cargo/issues/5457>
    out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn read_folder_recursive<F>(path: PathBuf, cb: &mut F)
where
    F: FnMut(PathBuf),
{
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            read_folder_recursive(path, cb);
        } else {
            cb(path);
        }
    }
}

fn sign_sha256sums(path: &PathBuf) {
    let key_base64 = std::env::var("TAURI_SIGNING_PRIVATE_KEY").unwrap_or_default();
    let password = std::env::var("TAURI_SIGNING_PRIVATE_KEY_PASSWORD").unwrap_or_default();

    let sig_path = path.with_extension("sig");

    // No signing key available (e.g. forks / self-built releases): emit an
    // unsigned placeholder instead of failing the build. The runtime integrity
    // check recognizes this marker and falls back to checksum-only validation.
    if key_base64.trim().is_empty() {
        println!("cargo:warning=TAURI_SIGNING_PRIVATE_KEY not set; producing an UNSIGNED build.");
        std::fs::write(&sig_path, slu_utils::signature::UNSIGNED_MARKER)
            .expect("Failed to write placeholder signature");
        return;
    }

    let data = std::fs::read(path).expect("Failed to read SHA256SUMS file");
    let signature = sign_minisign(&data, &key_base64, password).expect("Failed to sign data");

    std::fs::write(&sig_path, signature).expect("Failed to write signature");
}
