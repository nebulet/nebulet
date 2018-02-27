
#![feature(termination_trait)]

use std::process::Command;
use std::io::{self, Read};
use std::fs::File;

const TARGET: &'static str = "x86_64-nebulet";

fn main() -> io::Result<()> {
    let exit_status = build_image()?;
    if !exit_status.success() { panic!("something went wrong with bootimage"); }

    run_qemu()?;

    Ok(())
}

fn run_qemu() -> io::Result<std::process::ExitStatus> {
    let mut file = File::open("/proc/sys/kernel/osrelease")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let qemu_path = if contents.contains("Microsoft") {
        // Running in WSL
        "/mnt/c/Program Files/qemu/qemu-system-x86_64.exe"
    } else {
        "qemu-system-x86_64"
    };

    Command::new(qemu_path)
        .args(&["-hda", "bootimage.bin"])
        .args(&["-serial", "stdio"])
        // .args(&["-d", "int", "-no-reboot"])
        .status()
}

fn build_image() -> io::Result<std::process::ExitStatus> {
    Command::new("bootimage")
        .arg("--target").arg(TARGET)
        .arg("--release")
        .status()
}