use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{exit, Command, ExitStatus};

fn build_wat(wat_file: &Path, wasm_file: &Path) -> io::Result<ExitStatus> {
    Command::new("wat2wasm")
        .arg(wat_file)
        .arg("-o")
        .arg(wasm_file)
        .status()
}

fn main() {
    for wat_file in fs::read_dir("wasm").expect("Missing wasm dir") {
        let wat_file = wat_file.unwrap();
        let name = wat_file.file_name();
        let mut wasm_file = PathBuf::from("src/tests/wasmtests");
        wasm_file.push(name);
        wasm_file.set_extension("wasm");
        let r = build_wat(&wat_file.path(), &wasm_file).unwrap();
        if !r.success() {
            match r.code() {
                Some(x) => eprintln!("wat2wasm exited with status {}", x),
                None => eprintln!("wat2wasm terminated by signal"),
            };
            exit(1);
        }
    }
}
