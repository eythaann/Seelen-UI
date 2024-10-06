// #!/usr/bin/env rust-script
use std::fs;

fn main() {
    let dist = std::path::PathBuf::from("dist");
    let temp = std::path::PathBuf::from(".dist");
    if dist.exists() {
        fs::rename(&dist, temp).unwrap();
    }
    fs::create_dir(dist).unwrap();
}
