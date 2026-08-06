#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/blcp nix shell "github:nix-community/fenix?rev=7b33c6466f781cd699fe250c5b69dc4193da67a7#minimal.toolchain" nixpkgs#gcc -c cargo -q -Zscript
---cargo
[package]
name = "check-budget-ledger-purity"
version = "0.1.0"
edition = "2024"
---

// Guard that the budget-ledger-core crate stays a pure side-effect-free core.
//
// The crate owns deterministic reservation and settlement decisions. It must
// not reach for store, filesystem, environment, clock, process, network,
// provider, CLI, event-hashing, or product-authority facilities. Any such
// token in the crate source or an impure dependency in its manifest is a
// boundary violation and fails this check.

use std::fs;
use std::path::Path;

const CRATE_DEFAULT: &str = "crates/budget-ledger-core";

const FORBIDDEN_SOURCE: &[&str] = &[
    "std::fs",
    "std::env",
    "std::process",
    "std::time",
    "std::net",
    "std::thread",
    "std::sync",
    "std::io",
    "read_dir",
    "read_to_string",
    "std::fs::read",
    "std::fs::write",
    "std::fs::File",
    "Command",
    "tokio",
    "reqwest",
    "redb",
    "chrono",
    "rand",
    "serial_test",
];

// Dependency crates that would pull I/O, clock, concurrency, or store
// facilities into the core.
const FORBIDDEN_DEPS: &[&str] = &[
    "tokio",
    "reqwest",
    "redb",
    "chrono",
    "rand",
    "rayon",
    "parking_lot",
    "async-trait",
    "futures",
];

const ALLOWED_DEPS: &[&str] = &["serde"];

fn main() {
    let root_arg = std::env::args().nth(1).unwrap_or_else(|| CRATE_DEFAULT.to_string());
    let crate_dir = Path::new(&root_arg);
    let src_path = crate_dir.join("src/lib.rs");
    let manifest_path = crate_dir.join("Cargo.toml");

    let lib = fs::read_to_string(&src_path).expect("budget-ledger-core src/lib.rs must exist");
    let manifest = fs::read_to_string(&manifest_path).expect("budget-ledger-core Cargo.toml must exist");

    let mut failures: Vec<String> = Vec::new();

    for token in FORBIDDEN_SOURCE {
        if lib.contains(token) {
            failures.push(format!("forbidden source token in src/lib.rs: {token}"));
        }
    }

    // Dependency guard: every hard dependency under [dependencies] must be
    // allowlisted. Optional/dev/build sections are not part of the shipped core.
    let deps_section = manifest
        .split("[dependencies]")
        .nth(1)
        .map(|section| section.split('[').next().unwrap_or_default())
        .unwrap_or_default();
    for forbidden in FORBIDDEN_DEPS {
        let needle = format!("{forbidden} =");
        if deps_section.contains(&needle) {
            failures.push(format!("forbidden dependency in Cargo.toml: {forbidden}"));
        }
    }
    for name in ALLOWED_DEPS {
        let needle = format!("{name} =");
        if !deps_section.contains(&needle) {
            failures.push(format!("expected dependency missing from Cargo.toml: {name}"));
        }
    }

    if failures.is_empty() {
        println!("budget-ledger-core purity guard: PASS");
        return;
    }
    for failure in &failures {
        eprintln!("budget-ledger-core purity guard: {failure}");
    }
    eprintln!("budget-ledger-core purity guard: FAIL");
    std::process::exit(1);
}
