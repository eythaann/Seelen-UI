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
    std::fs::write("./gen/SHA256SUMS", out).unwrap();
    std::fs::write("./gen/SHA256SUMS.sig", "NOT SIGNED YET").unwrap();
    tauri_build::build();
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
