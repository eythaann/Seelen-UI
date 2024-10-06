// #!/usr/bin/env rust-script
use std::fs;

fn main() {
    let _ = fs::rename("dist", ".dist");
    let _ = fs::create_dir("dist");
}
