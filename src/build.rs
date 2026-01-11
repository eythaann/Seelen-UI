use std::{fs::create_dir, path::PathBuf};

use sha2::Digest;

fn main() {
    let _ = create_dir("gen");

    let mut out = String::new();
    read_folder_recursive(PathBuf::from("static"), &mut |path| {
        let file = std::fs::read(&path).unwrap();
        let hash = sha2::Sha256::digest(file);

        let path = path.display().to_string().replace("\\", "/");
        out.push_str(&format!("{:x}  {}\n", hash, path));
    });

    let target_dir = target_dir();
    let sums_path = target_dir.join("SHA256SUMS");
    std::fs::write(&sums_path, out).unwrap();

    if !cfg!(debug_assertions) {
        sign_sha256sums(&sums_path);
    } else {
        std::fs::write(
            sums_path.with_extension(".sig"),
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
    use base64::Engine;
    use minisign::SecretKeyBox;
    use std::io::{Cursor, Write};

    let key_base64 =
        std::env::var("TAURI_SIGNING_PRIVATE_KEY").expect("TAURI_SIGNING_PRIVATE_KEY missing");
    let password = std::env::var("TAURI_SIGNING_PRIVATE_KEY_PASSWORD")
        .expect("TAURI_SIGNING_PRIVATE_KEY_PASSWORD missing");

    let key_bytes = base64::engine::general_purpose::STANDARD
        .decode(&key_base64)
        .expect("Failed to decode base64 secret key");
    let key_str = String::from_utf8(key_bytes).expect("Secret key is not valid UTF-8");

    let sk_box = SecretKeyBox::from_string(&key_str).expect("Invalid secret key format");
    let secret_key = sk_box
        .into_secret_key(Some(password))
        .expect("Failed to decrypt secret key - invalid password");

    // Read the data to sign
    let data = std::fs::read(path).expect("Failed to read SHA256SUMS file");
    let data_reader = Cursor::new(&data);

    // Sign the data
    let signature_box =
        minisign::sign(None, &secret_key, data_reader, None, None).expect("Failed to sign data");

    // Write the signature to a .sig file
    let sig_path = path.with_extension("sig");
    let mut file = std::fs::File::create(&sig_path).expect("Failed to create signature file");
    file.write_all(signature_box.to_string().as_bytes())
        .expect("Failed to write signature");

    println!(
        "cargo:warning=Created signature file: {}",
        sig_path.display()
    );
}
