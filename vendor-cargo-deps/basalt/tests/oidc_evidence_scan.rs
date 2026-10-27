use std::fs;
use std::path::{Path, PathBuf};

const DURABLE_ROOTS: [&str; 3] = [
    "docs",
    ".cairn/archive/2026-08-18-exchange-oidc-for-ucan-capabilities",
    "contracts/oidc-exchange/generated",
];
const PRIVATE_KEY_MARKER: &str = "-----BEGIN PRIVATE KEY-----";
const OIDC_BEARER_FIELD: &str = "\"oidc_token\":";
const UCAN_BEARER_FIELD: &str = "\"ucan_token\":";
const PROVIDER_CREDENTIAL_FIELD: &str = "\"provider_credential\":";
const TOKEN_DIGEST_FIELD: &str = "\"token_digest\":";

fn source_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn collect_files(root: &Path, output: &mut Vec<PathBuf>) {
    if root.is_file() {
        output.push(root.to_path_buf());
        return;
    }
    let mut entries: Vec<_> = fs::read_dir(root)
        .expect("durable evidence directory")
        .map(|entry| entry.expect("durable evidence entry").path())
        .collect();
    entries.sort();
    for entry in entries {
        collect_files(entry.as_path(), output);
    }
}

fn prohibited_marker(text: &str) -> Option<&'static str> {
    [
        PRIVATE_KEY_MARKER,
        OIDC_BEARER_FIELD,
        UCAN_BEARER_FIELD,
        PROVIDER_CREDENTIAL_FIELD,
        TOKEN_DIGEST_FIELD,
    ]
    .into_iter()
    .find(|marker| text.contains(marker))
}

#[test]
fn durable_oidc_outputs_contain_no_bearer_private_or_correlation_material() {
    let root = source_root();
    let mut files = Vec::new();
    for relative in DURABLE_ROOTS {
        collect_files(root.join(relative).as_path(), &mut files);
    }
    assert!(!files.is_empty());
    for file in files {
        let text = fs::read_to_string(file.as_path()).expect("UTF-8 durable evidence");
        assert_eq!(
            prohibited_marker(text.as_str()),
            None,
            "prohibited evidence marker in {}",
            file.display()
        );
    }
}

#[test]
fn evidence_scan_rejects_each_prohibited_marker() {
    for marker in [
        PRIVATE_KEY_MARKER,
        OIDC_BEARER_FIELD,
        UCAN_BEARER_FIELD,
        PROVIDER_CREDENTIAL_FIELD,
        TOKEN_DIGEST_FIELD,
    ] {
        assert_eq!(prohibited_marker(marker), Some(marker));
    }
}
