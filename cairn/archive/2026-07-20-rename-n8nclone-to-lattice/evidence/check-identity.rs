#!/usr/bin/env -S nix develop . -c cargo -q -Zscript

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

const SELF_TEST_ARG: &str = "--self-test";
const SUCCESS_EXIT_CODE: u8 = 0;
const FAILURE_EXIT_CODE: u8 = 1;
const EXPECTED_SELF_TEST_ARG_COUNT: usize = 2;
const LEGACY_NAME_LOWER: &str = "n8nclone";
const CURRENT_ROOTS: &[&str] = &[
    "Cargo.lock",
    "Cargo.toml",
    "README.md",
    "cairn/specs",
    "examples",
    "flake.nix",
    "octet-baseline.json",
    "scripts",
    "src",
    "tests",
];
const TEXT_EXTENSIONS: &[&str] = &["json", "lock", "md", "ncl", "nix", "rs", "toml"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Finding {
    path: PathBuf,
    line_number: usize,
    line: String,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::from(SUCCESS_EXIT_CODE),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(FAILURE_EXIT_CODE)
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() == EXPECTED_SELF_TEST_ARG_COUNT && args[1] == SELF_TEST_ARG {
        self_test()?;
        println!("identity audit self-test passed");
        return Ok(());
    }
    if args.len() != 1 {
        return Err(format!("usage: {} [{SELF_TEST_ARG}]", args[0]));
    }

    let root = env::current_dir().map_err(|error| format!("failed to read current directory: {error}"))?;
    validate_canonical_surfaces(&root)?;
    let mut findings = Vec::new();
    for relative in CURRENT_ROOTS {
        inspect_path(&root, &root.join(relative), &mut findings)?;
    }
    if findings.is_empty() {
        println!("identity audit passed: only allowlisted compatibility and evidence identities remain");
        return Ok(());
    }
    for finding in &findings {
        eprintln!("{}:{}: {}", finding.path.display(), finding.line_number, finding.line.trim());
    }
    Err(format!("identity audit found {} stale public reference(s)", findings.len()))
}

// r[verify lattice.project_identity.canonical.scenario.stale-branding]
// r[verify lattice.project_identity.stable-identities.scenario.cairn-history]
fn validate_canonical_surfaces(root: &Path) -> Result<(), String> {
    require_text(root, "Cargo.toml", "[package]\nname = \"lattice\"")?;
    require_text(root, "Cargo.toml", "[[bin]]\nname = \"lattice\"")?;
    require_text(root, "flake.nix", "lattice = craneLib.buildPackage")?;
    require_text(root, "README.md", "# Lattice")?;
    require_text(root, "README.md", "lattice --config")?;
    Ok(())
}

fn require_text(root: &Path, relative: &str, needle: &str) -> Result<(), String> {
    let path = root.join(relative);
    let text = fs::read_to_string(&path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    if text.contains(needle) {
        Ok(())
    } else {
        Err(format!("{} is missing canonical identity text: {needle}", path.display()))
    }
}

fn inspect_path(root: &Path, path: &Path, findings: &mut Vec<Finding>) -> Result<(), String> {
    if path.is_dir() {
        let mut entries: Vec<PathBuf> = fs::read_dir(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?
            .map(|entry| {
                entry
                    .map(|value| value.path())
                    .map_err(|error| format!("failed to inspect {}: {error}", path.display()))
            })
            .collect::<Result<_, _>>()?;
        entries.sort();
        for entry in entries {
            inspect_path(root, &entry, findings)?;
        }
        return Ok(());
    }
    if !is_text_path(path) {
        return Ok(());
    }

    let relative = path
        .strip_prefix(root)
        .map_err(|error| format!("failed to relativize {}: {error}", path.display()))?;
    let text = fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    for (line_index, line) in text.lines().enumerate() {
        if line.to_ascii_lowercase().contains(LEGACY_NAME_LOWER) && !is_allowed(relative, line) {
            findings.push(Finding {
                path: relative.to_path_buf(),
                line_number: line_index.saturating_add(1),
                line: line.to_string(),
            });
        }
    }
    Ok(())
}

fn is_allowed(path: &Path, line: &str) -> bool {
    if line.contains("r[") {
        return true;
    }
    if path == Path::new("cairn/specs/project-identity/spec.md")
        || path == Path::new("src/config/identity.rs")
        || path == Path::new("tests/test_identity.rs")
    {
        return true;
    }
    if path == Path::new("src/triggers.rs") && line.contains("n8nclone-webhook-signature:v1") {
        return true;
    }
    path == Path::new("src/credentials.rs")
        && (line.contains("n8nclone-local-vault-stream-v1") || line.contains("n8nclone-local-vault-integrity-v1"))
}

fn is_text_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| TEXT_EXTENSIONS.contains(&extension))
}

fn self_test() -> Result<(), String> {
    if !is_allowed(Path::new("src/triggers.rs"), "const DOMAIN: &[u8] = b\"n8nclone-webhook-signature:v1\";") {
        return Err("stable webhook domain was not allowlisted".to_string());
    }
    if is_allowed(Path::new("README.md"), "run n8nclone --config config.ncl") {
        return Err("stale operator command was incorrectly allowlisted".to_string());
    }
    if is_allowed(Path::new("src/runtime.rs"), "const NAME: &str = \"n8nclone\";") {
        return Err("stale runtime branding was incorrectly allowlisted".to_string());
    }
    Ok(())
}
