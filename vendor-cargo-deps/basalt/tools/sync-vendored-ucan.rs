#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/basalt-sync-vendored-ucan nix shell "github:nix-community/fenix?rev=4e49ef62eedaa5c149d62b63b3f53844e1cc45d7#complete.toolchain" nixpkgs#gcc nixpkgs#git nixpkgs#gnutar nixpkgs#b3sum -c cargo -q -Zscript

#[path = "sync_vendored_ucan/filesystem.rs"]
mod filesystem;
#[path = "sync_vendored_ucan/model.rs"]
mod model;
#[path = "sync_vendored_ucan/process.rs"]
mod process;
#[path = "sync_vendored_ucan/shell.rs"]
mod shell;
#[cfg(test)]
#[path = "sync_vendored_ucan/tests.rs"]
mod tests;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments = env::args().collect::<Vec<_>>();
    let root = match env::current_dir() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("error: cannot read current directory: {error}");
            return ExitCode::FAILURE;
        }
    };
    let result = model::parse_mode(&arguments).and_then(|mode| shell::run(&root, mode));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
