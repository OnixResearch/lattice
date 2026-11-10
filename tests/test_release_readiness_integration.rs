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
    let policy = json!({"name": "consumer-policy"});
    let policy_hash = release_inputs::json_hash(&policy);
    release_inputs::completion_inputs(
        &policy,
        &[
            "native-cairn".into(),
            "mcp".into(),
            "--call".into(),
            "cairn_validate".into(),
        ],
        &json!({"isError": false, "structuredContent": {"valid": valid, "mcp": {"read_only": true}}}),
        &json!({"command": ["native-cairn", "mcp", "--call", "cairn_validate"], "exit_code": exit_code, "stdout_hash": hash, "stderr_hash": hash, "native_binary_hash": hash, "subject_hash": hash, "policy_hash": policy_hash}),
        &hash,
    )
}

#[test]
fn policy_declares_the_observation_output_paths() -> TestResult {
    let policy: Value = serde_json::from_str(include_str!("../cairn-policy/generated/cairn-policy.json"))?;
    let selected = &policy["release_readiness_policy"];
    assert_eq!(selected["mcp_completion_contract"], release_inputs::CONTRACT_PATH);
    assert_eq!(selected["mcp_completion_results"], release_inputs::RESULTS_PATH);
    Ok(())
}

#[test]
fn mcp_completion_status_requires_both_native_success_and_passing_verdict() -> TestResult {
    // r[verify lattice.release_integration.inputs]
    // r[verify lattice.release_integration.inputs.mcp]
    let passed = observed_inputs(true, 0)?;
    assert!(passed.passed);
    assert_eq!(passed.results[0]["status"], "passed");
    assert_eq!(passed.results[0]["artifacts"][1]["path"], release_inputs::STDERR_PATH);
    let subject_hash = release_inputs::json_hash(&json!("observed-test-bytes"));
    assert_eq!(passed.results[0]["subject_hash"], subject_hash);
    assert_eq!(passed.contract["final_subject_hash"], subject_hash);
    assert_eq!(passed.results[0]["policy_hash"], release_inputs::json_hash(&json!({"name": "consumer-policy"})));
    assert_eq!(passed.contract["required_checks"][0]["command_hash"], passed.results[0]["command_hash"]);
    assert_eq!(passed.contract["registry_hash"], passed.results[0]["registry_hash"]);
    assert_eq!(passed.contract["selection_hash"], passed.results[0]["selection_hash"]);
    assert_eq!(passed.contract["policy_hash"], passed.results[0]["policy_hash"]);
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
            &json!({"command": ["cairn"], "exit_code": 0, "stdout_hash": hash, "stderr_hash": hash, "native_binary_hash": hash, "subject_hash": hash, "policy_hash": release_inputs::json_hash(&json!({}))}),
            &hash
        )
        .is_err(),
        "a CLI-shaped verdict must not be relabelled as MCP execution evidence"
    );
    let valid_observation = json!({"command": ["cairn"], "exit_code": 0, "stdout_hash": hash, "stderr_hash": hash, "native_binary_hash": hash, "subject_hash": hash, "policy_hash": release_inputs::json_hash(&json!({}))});
    assert!(
        release_inputs::completion_inputs(
            &json!({}),
            &[],
            &json!({"isError": false, "structuredContent": {"valid": true, "mcp": {"read_only": true}}}),
            &valid_observation,
            &hash
        )
        .is_err(),
        "an empty observed command was accepted"
    );
    assert!(
        release_inputs::completion_inputs(
            &json!({}),
            &["".into()],
            &json!({"isError": false, "structuredContent": {"valid": true, "mcp": {"read_only": true}}}),
            &valid_observation,
            &hash
        )
        .is_err(),
        "an empty command argument was accepted"
    );
    for validation in [
        json!({"isError": true, "structuredContent": {"valid": true, "mcp": {"read_only": true}}}),
        json!({"structuredContent": {"valid": true, "mcp": {"read_only": true}}}),
        json!({"isError": false}),
        json!({"isError": false, "structuredContent": {"mcp": {"read_only": true}}}),
        json!({"isError": false, "structuredContent": {"valid": "yes", "mcp": {"read_only": true}}}),
        json!({"isError": false, "structuredContent": {"valid": true, "mcp": {"read_only": false}}}),
    ] {
        assert!(
            release_inputs::completion_inputs(&json!({}), &["cairn".into()], &validation, &valid_observation, &hash)
                .is_err(),
            "invalid MCP envelope was accepted: {validation}"
        );
    }
    let mut missing_exit = valid_observation.clone();
    missing_exit.as_object_mut().ok_or("observation object")?.remove("exit_code");
    assert!(
        release_inputs::completion_inputs(
            &json!({}),
            &["cairn".into()],
            &json!({"isError": false, "structuredContent": {"valid": true, "mcp": {"read_only": true}}}),
            &missing_exit,
            &hash
        )
        .is_err(),
        "an observation without an exit code must not be accepted"
    );
    let mut observation = valid_observation.clone();
    observation.as_object_mut().ok_or("observation object")?.remove("stderr_hash");
    assert!(
        release_inputs::completion_inputs(
            &json!({}),
            &["cairn".into()],
            &json!({"isError": false, "structuredContent": {"valid": true, "mcp": {"read_only": true}}}),
            &observation,
            &hash
        )
        .is_err(),
        "an observation without a retained stderr identity must not be accepted"
    );
    Ok(())
}

#[test]
fn completion_inputs_bind_the_observed_command() -> TestResult {
    // r[verify lattice.release_integration.inputs.mcp]
    let policy = json!({});
    let hash = release_inputs::json_hash(&json!("observed-test-bytes"));
    let validation = json!({"isError": false, "structuredContent": {"valid": true, "mcp": {"read_only": true}}});
    let command = vec![
        "cairn".to_string(),
        "mcp".to_string(),
        "--call".to_string(),
        "cairn_validate".to_string(),
    ];
    let observation = json!({
        "command": ["cairn", "mcp", "--call", "cairn_validate"],
        "exit_code": 0,
        "stdout_hash": hash,
        "stderr_hash": hash,
        "native_binary_hash": hash,
        "subject_hash": hash,
        "policy_hash": release_inputs::json_hash(&policy),
    });
    assert!(release_inputs::completion_inputs(&policy, &command, &validation, &observation, &hash).is_ok());
    let mut mismatched = observation.clone();
    mismatched["command"] = json!(["cairn", "mcp", "--call", "cairn_status"]);
    assert!(
        release_inputs::completion_inputs(&policy, &command, &validation, &mismatched, &hash).is_err(),
        "an observation for a different command was accepted"
    );
    let mut missing = observation;
    missing.as_object_mut().ok_or("observation object")?.remove("command");
    assert!(
        release_inputs::completion_inputs(&policy, &command, &validation, &missing, &hash).is_err(),
        "an observation without a command was accepted"
    );
    Ok(())
}

#[test]
fn observation_paths_share_the_output_root_and_are_distinct() {
    let paths = [
        release_inputs::CONTRACT_PATH,
        release_inputs::RESULTS_PATH,
        release_inputs::VALIDATION_PATH,
        release_inputs::STDERR_PATH,
        release_inputs::SOURCE_MANIFEST_PATH,
        release_inputs::OBSERVATION_PATH,
    ];
    for path in paths {
        assert!(path.starts_with(release_inputs::OUTPUT_ROOT), "{path}");
        let parsed = std::path::Path::new(path);
        assert!(parsed.is_relative(), "{path}");
        assert!(parsed.components().all(|component| matches!(component, std::path::Component::Normal(_))), "{path}");
    }
    for (index, path) in paths.iter().enumerate() {
        for other in &paths[index + 1..] {
            assert_ne!(path, other);
        }
    }
}

#[test]
fn native_size_bound_rejects_oversized_executables() {
    assert!(release_inputs::check_native_size(release_inputs::MAX_NATIVE_BYTES).is_ok());
    let error = release_inputs::check_native_size(release_inputs::MAX_NATIVE_BYTES + 1).expect_err("native size limit");
    assert!(error.contains(&(release_inputs::MAX_NATIVE_BYTES + 1).to_string()));
    assert!(error.contains(&release_inputs::MAX_NATIVE_BYTES.to_string()));
}

#[test]
fn json_hash_preserves_array_order() {
    assert_ne!(release_inputs::json_hash(&json!({"a": [1, 2]})), release_inputs::json_hash(&json!({"a": [2, 1]})));
}

#[test]
fn json_hash_ignores_object_key_order() {
    let mut left = serde_json::Map::new();
    left.insert("a".to_string(), json!(1));
    left.insert("b".to_string(), json!(2));
    let mut right = serde_json::Map::new();
    right.insert("b".to_string(), json!(2));
    right.insert("a".to_string(), json!(1));
    assert_eq!(release_inputs::json_hash(&Value::Object(left)), release_inputs::json_hash(&Value::Object(right)));
}

#[test]
fn json_hash_distinguishes_null_from_absent() {
    assert_ne!(release_inputs::json_hash(&json!({"a": null})), release_inputs::json_hash(&json!({})));
}

#[test]
fn json_hash_ignores_nested_object_key_order() {
    let left = json!({"outer": {"a": 1, "b": 2}});
    let right = json!({"outer": {"b": 2, "a": 1}});
    assert_eq!(release_inputs::json_hash(&left), release_inputs::json_hash(&right));
}

#[test]
fn json_hash_preserves_nested_array_order() {
    let left = json!({"outer": [1, 2]});
    let right = json!({"outer": [2, 1]});
    assert_ne!(release_inputs::json_hash(&left), release_inputs::json_hash(&right));
}

#[test]
fn manifest_bounds_reject_oversized_manifests() {
    assert!(release_inputs::check_manifest_bounds(0, 0).is_ok());
    assert!(
        release_inputs::check_manifest_bounds(release_inputs::MAX_MANIFEST_FILES, release_inputs::MAX_MANIFEST_BYTES)
            .is_ok()
    );
    let file_error =
        release_inputs::check_manifest_bounds(release_inputs::MAX_MANIFEST_FILES + 1, 0).expect_err("file-count limit");
    assert!(file_error.contains(&(release_inputs::MAX_MANIFEST_FILES + 1).to_string()));
    let byte_error =
        release_inputs::check_manifest_bounds(0, release_inputs::MAX_MANIFEST_BYTES + 1).expect_err("byte limit");
    assert!(byte_error.contains(&(release_inputs::MAX_MANIFEST_BYTES + 1).to_string()));
    assert!(
        release_inputs::check_manifest_bounds(
            release_inputs::MAX_MANIFEST_FILES + 1,
            release_inputs::MAX_MANIFEST_BYTES + 1
        )
        .is_err()
    );
}

#[test]
fn manifest_path_keys_preserve_exact_utf8_paths_and_reject_non_utf8() {
    assert_eq!(
        release_inputs::manifest_path_key(std::path::Path::new("cairn-policy/default.ncl")).expect("UTF-8 path"),
        "cairn-policy/default.ncl"
    );
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let path = std::path::Path::new(std::ffi::OsStr::from_bytes(b"cairn-policy/\xff"));
        assert!(
            release_inputs::manifest_path_key(path).is_err(),
            "a non-UTF-8 path must not be lossily collapsed into the manifest"
        );
    }
}

#[test]
fn completion_inputs_reject_malformed_identity_hashes() -> TestResult {
    let hash = release_inputs::json_hash(&json!("observed-test-bytes"));
    let validation = json!({"isError": false, "structuredContent": {"valid": true, "mcp": {"read_only": true}}});
    for field in [
        "stdout_hash",
        "stderr_hash",
        "native_binary_hash",
        "subject_hash",
        "policy_hash",
    ] {
        let mut observation = json!({"command": ["cairn"], "exit_code": 0, "stdout_hash": hash, "stderr_hash": hash, "native_binary_hash": hash, "subject_hash": hash, "policy_hash": release_inputs::json_hash(&json!({}))});
        observation[field] = json!("not-a-blake3-digest");
        assert!(
            release_inputs::completion_inputs(&json!({}), &["cairn".into()], &validation, &observation, &hash).is_err(),
            "malformed {field} was accepted"
        );
        observation.as_object_mut().ok_or("observation object")?.remove(field);
        assert!(
            release_inputs::completion_inputs(&json!({}), &["cairn".into()], &validation, &observation, &hash).is_err(),
            "missing {field} was accepted"
        );
    }
    let uppercase = "A".repeat(64);
    let uppercase_observation = json!({
        "command": ["cairn"],
        "exit_code": 0,
        "stdout_hash": uppercase,
        "stderr_hash": hash,
        "native_binary_hash": hash,
        "subject_hash": hash,
        "policy_hash": release_inputs::json_hash(&json!({})),
    });
    assert!(
        release_inputs::completion_inputs(&json!({}), &["cairn".into()], &validation, &uppercase_observation, &hash)
            .is_err(),
        "an uppercase identity was accepted"
    );
    let uppercase_subject_observation = json!({
        "command": ["cairn"],
        "exit_code": 0,
        "stdout_hash": hash,
        "stderr_hash": hash,
        "native_binary_hash": hash,
        "subject_hash": uppercase,
        "policy_hash": release_inputs::json_hash(&json!({})),
    });
    assert!(
        release_inputs::completion_inputs(
            &json!({}),
            &["cairn".into()],
            &validation,
            &uppercase_subject_observation,
            &uppercase
        )
        .is_err(),
        "an uppercase subject identity was accepted"
    );
    let mismatched = json!({
        "command": ["cairn"],
        "exit_code": 0,
        "stdout_hash": hash,
        "stderr_hash": hash,
        "native_binary_hash": hash,
        "subject_hash": release_inputs::json_hash(&json!("different-subject")),
        "policy_hash": release_inputs::json_hash(&json!({})),
    });
    assert!(
        release_inputs::completion_inputs(&json!({}), &["cairn".into()], &validation, &mismatched, &hash).is_err(),
        "a mismatched observation subject was accepted"
    );
    let mut mismatched_policy = mismatched.clone();
    mismatched_policy["subject_hash"] = json!(hash);
    mismatched_policy["policy_hash"] = json!(release_inputs::json_hash(&json!("different-policy")));
    assert!(
        release_inputs::completion_inputs(&json!({}), &["cairn".into()], &validation, &mismatched_policy, &hash)
            .is_err(),
        "a mismatched observation policy was accepted"
    );
    assert!(
        release_inputs::completion_inputs(
            &json!({}),
            &["cairn".into()],
            &validation,
            &json!({"command": ["cairn"], "exit_code": 0, "stdout_hash": hash, "stderr_hash": hash, "native_binary_hash": hash, "subject_hash": hash, "policy_hash": release_inputs::json_hash(&json!({}))}),
            "not-a-blake3-digest"
        )
        .is_err(),
        "malformed subject hash was accepted"
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
