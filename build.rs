use std::process::Command;
use std::env::remove_var;

fn main() {
    // Remove sysroot flag... well remove all of them for now.
    remove_var("RUSTFLAGS");

    println!("cargo:rerun-if-changed=userspace");

    let ret = Command::new("cargo")
        .args(&[
            "build",
            "--manifest-path", "userspace/Cargo.toml",
            "--target", "wasm32-unknown-unknown",
            "--release",
        ])
        .status()
        .expect("Failed to build userspace");

    assert!(ret.success());
}
