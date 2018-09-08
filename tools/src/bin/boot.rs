use std::fs::File;
use std::io::{self, Read};
use std::process::Command;

fn main() -> io::Result<()> {
    // let exit_status = build_image()?;
    // if !exit_status.success() { panic!("something went wrong with bootimage"); }

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
        .args(&["-hda", "bootimage.bin", "-s"])
        .args(&["-serial", "stdio"])
        // .args(&["-chardev", "socket,id=qemu-monitor,host=localhost,port=7777,server,nowait,telnet", "-mon", "qemu-monitor,mode=readline"])
        // .args(&["-d", "int", "-no-reboot"])
        // .arg("-S")
        .status()
}

// fn build_image() -> io::Result<std::process::ExitStatus> {
//     Command::new("bootimage")
//         .arg("build")
//         // .arg("--release")
//         // .arg("--build-bootloader")
//         // .arg("--git").arg("https://github.com/rust-osdev/bootloader")
//         .status()
// }
