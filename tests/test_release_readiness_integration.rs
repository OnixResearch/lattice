#[path = "support/cairn.rs"]
mod cairn;
#[path = "../scripts/release_readiness/core.rs"]
mod release_inputs;

use cairn::Fixture;
use cairn::TestResult;
use serde_json::Value;
use serde_json::json;

fn observed_inputs(valid: bool, exit_code: i32) -> Result<release_inputs::CompletionInputs, String> {
    let hash = release_inputs::json_hash(&json!("observed-test-bytes"));
    release_inputs::completion_inputs(
        &json!({"name": "consumer-policy"}),
        &[
            "native-cairn".into(),
            "mcp".into(),
            "--call".into(),
            "cairn_validate".into(),
        ],
        &json!({"isError": false, "structuredContent": {"valid": valid, "mcp": {"read_only": true}}}),
        &json!({"exit_code": exit_code, "stdout_hash": hash, "native_binary_hash": hash}),
        &hash,
    )
}

#[test]
fn mcp_completion_status_requires_both_native_success_and_passing_verdict() -> TestResult {
    // r[verify lattice.release_integration.inputs]
    // r[verify lattice.release_integration.inputs.mcp]
    let passed = observed_inputs(true, 0)?;
    assert!(passed.passed);
    assert_eq!(passed.results[0]["status"], "passed");
    for (valid, exit_code) in [(false, 0), (true, 1), (false, 124)] {
        let failed = observed_inputs(valid, exit_code)?;
        assert!(!failed.passed);
        assert_eq!(failed.results[0]["status"], "failed");
        assert_eq!(
            passed.contract, failed.contract,
            "a failed observation must not change the admitted check selection"
        );
    }
    let hash = release_inputs::json_hash(&json!("observed-test-bytes"));
    assert!(
        release_inputs::completion_inputs(
            &json!({}),
            &["cairn".into()],
            &json!({"valid": true}),
            &json!({"exit_code": 0, "stdout_hash": hash, "native_binary_hash": hash}),
            &hash
        )
        .is_err(),
        "a CLI-shaped verdict must not be relabelled as MCP execution evidence"
    );
    Ok(())
}

#[test]
fn consumer_canonicalization_retains_sequence_and_null_boundaries() -> TestResult {
    // r[verify lattice.release_integration.inputs.corpus]
    let fixture = Fixture::new()?;
    let bytes = include_bytes!("../fixtures/release-readiness/canonicalization.json");
    fixture.write("consumer-corpus.json", bytes)?;
    let (success, fresh) = fixture.run(&["preserves-corpus", "update", "--path", "consumer-corpus.json"])?;
    assert!(success);
    assert_eq!(fresh["valid"], true);
    assert_eq!(fresh["stale_count"], 0);
    let rows = fresh["cases"].as_array().ok_or("corpus cases")?;
    assert_eq!(rows[0]["computed_hash"], rows[1]["computed_hash"]);
    assert_ne!(rows[0]["computed_hash"], rows[2]["computed_hash"]);
    assert_ne!(rows[0]["computed_hash"], rows[3]["computed_hash"]);
    let mut stale: Value = serde_json::from_slice(bytes)?;
    stale[0]["value"]["revision"] = json!(2);
    let stale_bytes = serde_json::to_vec(&stale)?;
    fixture.write("consumer-corpus.json", &stale_bytes)?;
    let (success, receipt) = fixture.run(&["preserves-corpus", "update", "--path", "consumer-corpus.json"])?;
    assert!(!success);
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["stale_count"], 1);
    assert_eq!(std::fs::read(fixture.root.path().join("consumer-corpus.json"))?, stale_bytes);
    Ok(())
}

#[test]
fn release_selection_lists_lattice_gates_not_foreign_producer_checks() -> TestResult {
    // r[verify lattice.release_integration.selection.gates]
    let fixture = Fixture::new()?;
    let (_, receipt) = fixture.run(&["release-readiness"])?;
    // Missing real MCP inputs must stay failed in an empty consumer root.
    let checks = receipt["checks"].as_array().ok_or("readiness checks")?;
    assert!(checks.iter().any(|check| check["name"] == "mcp_agent_smoke" && check["passed"] == false));
    assert_eq!(
        receipt["repository_gate_summary"]["required_commands"],
        json!([
            "scripts/check.rs",
            "octet-precommit-check",
            "nix flake check -L --option secret-key-files ''"
        ])
    );
    assert_eq!(receipt["determinism_replay"]["required"], true);
    assert_eq!(receipt["determinism_replay"]["included"], false);
    Ok(())
}
