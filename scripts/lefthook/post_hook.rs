// #!/usr/bin/env rust-script

fn main() {
    let temp = std::path::PathBuf::from(".dist");
    let dist = std::path::PathBuf::from("dist");
    if temp.exists() {
        std::fs::remove_dir(&dist).unwrap();
        std::fs::rename(temp, dist).unwrap();
    }
}
