extern crate tar;
extern crate walkdir;

use walkdir::WalkDir;
use std::process::Command;
use std::fs::File;
use std::io;

fn main() -> io::Result<()> {
    println!("Building userspace");

    Command::new("cargo")
        .args(&[
            "build",
            "--manifest-path", "userspace/Cargo.toml",
            "--release",
            "--target", "wasm32-unknown-unknown"
        ])
        .status()
        .expect("Failed to build userspace");

    println!("Building initfs.tar");

    let file = File::create("initfs.tar")?;
    let mut tar = tar::Builder::new(file);
    tar.mode(tar::HeaderMode::Deterministic);

    for entry in WalkDir::new("initfs").into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let name = entry.file_name().to_str().unwrap();
            println!("packaging from initfs: {}", name);
            tar.append_file(name, &mut File::open(entry.path())?)?;
        }
    }

    for entry in WalkDir::new("userspace/target/wasm32-unknown-unknown/release/")
                        .max_depth(1)
                         .into_iter()
                         .filter_map(|e| e.ok()) {
        if is_wasm(&entry) {
            assert!(entry.file_type().is_file());
            let name = entry.file_name().to_str().unwrap();
            println!("packaging: {}", name);
            tar.append_file(name, &mut File::open(entry.path())?)?;
        }
    }

    Ok(())
}

fn is_wasm(entry: &walkdir::DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.ends_with(".wasm"))
        .unwrap_or(false)
}