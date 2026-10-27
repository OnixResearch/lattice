#![cfg(feature = "cli")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn cli_binary() -> Option<&'static str> {
    let binary = option_env!("CARGO_BIN_EXE_basalt")?;
    if Path::new(binary).exists() {
        Some(binary)
    } else {
        // Cargo can carry the binary path without materializing the feature-gated binary
        // on the no-default-features rail.
        None
    }
}

fn unique_policy_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "basalt-{name}-{}-{unique}.json",
        std::process::id()
    ))
}

#[test]
fn cli_feature_exposes_enforce_smoke() {
    let Some(binary) = cli_binary() else {
        // The CLI smoke is asserted by the `--features cli` verification rail.
        return;
    };

    let output = Command::new(binary)
        .arg("enforce")
        .arg("--policy")
        .arg("contracts/generated/policy.json")
        .arg("--contract")
        .arg("document-write")
        .arg("--resource")
        .arg("did:key:z6Mkresource/docs/report")
        .arg("--ability")
        .arg("document/write")
        .arg("--capability")
        .arg("did:key:z6Mkresource/docs/report=document/write")
        .output()
        .expect("CLI process should run");

    assert!(output.status.success(), "CLI failed: {output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        r#"{"allowed":true,"reason":"allowed"}"#
    );
    assert!(output.stderr.is_empty(), "CLI stderr should be empty");
}

#[test]
fn cli_feature_reports_denied_receipt_for_policy_denial() {
    let Some(binary) = cli_binary() else {
        return;
    };

    let output = Command::new(binary)
        .arg("enforce")
        .arg("--policy")
        .arg("contracts/generated/policy.json")
        .arg("--contract")
        .arg("document-write")
        .arg("--resource")
        .arg("did:key:z6Mkresource/private/report")
        .arg("--ability")
        .arg("document/write")
        .arg("--capability")
        .arg("did:key:z6Mkresource/private/report=document/write")
        .output()
        .expect("CLI process should run");

    assert!(output.status.success(), "policy denial should be a receipt");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""allowed":false"#));
    assert!(stdout.contains("outside contract"));
}

#[test]
fn cli_feature_rejects_malformed_policy_file() {
    let Some(binary) = cli_binary() else {
        return;
    };
    let policy_dir = unique_policy_path("malformed-dir");
    fs::create_dir_all(&policy_dir).expect("policy dir should create");
    let policy_path = policy_dir.join("malformed.json");
    fs::write(&policy_path, "not-json").expect("policy fixture should write");
    let relative = policy_path
        .file_name()
        .expect("policy path has a file name")
        .to_string_lossy()
        .into_owned();

    let output = Command::new(binary)
        .arg("enforce")
        .arg("--policy-root")
        .arg(&policy_dir)
        .arg("--policy")
        .arg(&relative)
        .arg("--contract")
        .arg("document-write")
        .arg("--resource")
        .arg("did:key:z6Mkresource/docs/report")
        .arg("--ability")
        .arg("document/write")
        .arg("--capability")
        .arg("did:key:z6Mkresource/docs/report=document/write")
        .output()
        .expect("CLI process should run");

    fs::remove_dir_all(&policy_dir).expect("policy fixture should clean up");
    assert!(!output.status.success(), "malformed policy should fail");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("PolicyJson"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_feature_rejects_bad_capability_syntax() {
    let Some(binary) = cli_binary() else {
        return;
    };

    let output = Command::new(binary)
        .arg("enforce")
        .arg("--policy")
        .arg("contracts/generated/policy.json")
        .arg("--contract")
        .arg("document-write")
        .arg("--resource")
        .arg("did:key:z6Mkresource/docs/report")
        .arg("--ability")
        .arg("document/write")
        .arg("--capability")
        .arg("did:key:z6Mkresource/docs/report")
        .output()
        .expect("CLI process should run");

    assert!(
        !output.status.success(),
        "bad capability syntax should fail"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("expected RESOURCE=ABILITY"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_feature_writes_receipt_under_declared_output_root() {
    let Some(binary) = cli_binary() else {
        return;
    };
    let output_dir = unique_policy_path("output-root");
    fs::create_dir_all(&output_dir).expect("output dir should create");

    let output = Command::new(binary)
        .arg("enforce")
        .arg("--policy")
        .arg("contracts/generated/policy.json")
        .arg("--output-dir")
        .arg(&output_dir)
        .arg("--contract")
        .arg("document-write")
        .arg("--resource")
        .arg("did:key:z6Mkresource/docs/report")
        .arg("--ability")
        .arg("document/write")
        .arg("--capability")
        .arg("did:key:z6Mkresource/docs/report=document/write")
        .output()
        .expect("CLI process should run");

    let receipt = fs::read_to_string(output_dir.join("enforce-receipt.json"));
    fs::remove_dir_all(&output_dir).expect("output dir should clean up");
    assert!(output.status.success(), "CLI should succeed: {output:?}");
    let receipt = receipt.expect("receipt written under output root");
    assert!(receipt.contains(r#""allowed":true"#));
}

#[test]
fn cli_feature_rejects_traversal_outside_policy_root() {
    let Some(binary) = cli_binary() else {
        return;
    };

    let output = Command::new(binary)
        .arg("enforce")
        .arg("--policy")
        .arg("../../etc/passwd")
        .arg("--contract")
        .arg("document-write")
        .arg("--resource")
        .arg("did:key:z6Mkresource/docs/report")
        .arg("--ability")
        .arg("document/write")
        .arg("--capability")
        .arg("did:key:z6Mkresource/docs/report=document/write")
        .output()
        .expect("CLI process should run");

    assert!(
        !output.status.success(),
        "traversal outside the policy root should fail"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("InvalidPath"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
