#!/usr/bin/env -S nix develop . -c cargo -q -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.3"
serde = { version = "=1.0.228", features = ["derive"] }
serde_json = "=1.0.149"
---

#[path = "octet_baseline_migration.rs"]
mod migration;

fn main() -> std::process::ExitCode {
    match migration::run(&std::env::args().collect::<Vec<_>>()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
