// #!/usr/bin/env rust-script
use std::fs;

fn main() {
    let dist = std::path::PathBuf::from("dist");
    let temp = std::path::PathBuf::from(".dist");
    if dist.exists() && !temp.exists() {
        fs::rename(&dist, temp).unwrap();
    }
    if !dist.exists() {
        fs::create_dir(dist).unwrap();
    }
}
