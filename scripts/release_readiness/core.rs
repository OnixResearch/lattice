use serde_json::Value;
use serde_json::json;

pub const OUTPUT_ROOT: &str = "target/release-readiness";
pub const CONTRACT_PATH: &str = "target/release-readiness/mcp-contract.json";
pub const RESULTS_PATH: &str = "target/release-readiness/mcp-results.json";
pub const VALIDATION_PATH: &str = "target/release-readiness/validation.json";
pub const STDERR_PATH: &str = "target/release-readiness/validation.stderr";
pub const SOURCE_MANIFEST_PATH: &str = "target/release-readiness/source-manifest.json";
pub const OBSERVATION_PATH: &str = "target/release-readiness/observation.json";
pub const MAX_MANIFEST_FILES: usize = 50_000;
pub const MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024 * 1024;
pub const MAX_NATIVE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

pub fn check_native_size(bytes: u64) -> Result<(), String> {
    if bytes > MAX_NATIVE_BYTES {
        return Err(format!("native executable has {bytes} bytes; limit {MAX_NATIVE_BYTES}"));
    }
    Ok(())
}

pub fn check_manifest_bounds(file_count: usize, total_bytes: u64) -> Result<(), String> {
    if file_count > MAX_MANIFEST_FILES {
        return Err(format!("release source manifest has {file_count} files; limit {MAX_MANIFEST_FILES}"));
    }
    if total_bytes > MAX_MANIFEST_BYTES {
        return Err(format!("release source manifest has {total_bytes} bytes; limit {MAX_MANIFEST_BYTES}"));
    }
    Ok(())
}

pub struct CompletionInputs {
    pub contract: Value,
    pub results: Value,
    pub passed: bool,
}

pub fn json_hash(value: &Value) -> String {
    fn sorted(value: &Value) -> Value {
        match value {
            Value::Object(fields) => {
                let ordered = fields.iter().collect::<std::collections::BTreeMap<_, _>>();
                Value::Object(ordered.into_iter().map(|(key, child)| (key.clone(), sorted(child))).collect())
            }
            Value::Array(values) => Value::Array(values.iter().map(sorted).collect()),
            scalar => scalar.clone(),
        }
    }
    let bytes = serde_json::to_vec(&sorted(value)).expect("JSON values serialize without failure");
    blake3::hash(&bytes).to_hex().to_string()
}

// r[impl lattice.release_integration.inputs]
// r[impl lattice.release_integration.inputs.mcp]
pub fn completion_inputs(
    policy: &Value,
    command: &[String],
    validation: &Value,
    observation: &Value,
    subject_hash: &str,
) -> Result<CompletionInputs, String> {
    if command.is_empty() {
        return Err("observed command is empty".to_string());
    }
    if validation["isError"] != false || validation["structuredContent"]["mcp"]["read_only"] != true {
        return Err("native MCP validation envelope is absent or invalid".to_string());
    }
    let valid = validation["structuredContent"]["valid"]
        .as_bool()
        .ok_or("native MCP validation receipt lacks a boolean valid verdict")?;
    let exit_code = observation["exit_code"].as_i64().ok_or("native validation did not terminate with an exit code")?;
    let evidence_hash = observation["stdout_hash"].as_str().ok_or("native stdout identity is absent")?;
    let stderr_hash = observation["stderr_hash"].as_str().ok_or("native stderr identity is absent")?;
    let native_hash = observation["native_binary_hash"].as_str().ok_or("native executable identity is absent")?;
    let observed_subject =
        observation["subject_hash"].as_str().ok_or("native observation subject identity is absent")?;
    if observed_subject != subject_hash {
        return Err("native observation subject identity does not match the source manifest".to_string());
    }
    let policy_hash = json_hash(policy);
    let observed_policy = observation["policy_hash"].as_str().ok_or("native observation policy identity is absent")?;
    if observed_policy != policy_hash {
        return Err("native observation policy identity does not match the admitted policy".to_string());
    }
    for hash in [
        subject_hash,
        evidence_hash,
        stderr_hash,
        native_hash,
        policy_hash.as_str(),
    ] {
        if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("observed identities must be BLAKE3 hex digests".to_string());
        }
    }
    let passed = valid && exit_code == 0;
    let command_hash = json_hash(&json!(command));
    let checks = json!([{
        "check_id": "lattice-native-mcp-validation", "command": command,
        "command_hash": command_hash, "prerequisites": [],
    }]);
    let selection_hash = json_hash(&json!([{
        "check_id": "lattice-native-mcp-validation", "command_hash": command_hash, "prerequisites": [],
    }]));
    let registry_hash = json_hash(&json!({
        "registry_id": "lattice-release-mcp-observation-v1", "checks": checks,
        "native_binary_hash": native_hash,
        "boundary": "one observed native MCP lifecycle validation, not heavyweight release gates or source correctness",
    }));
    let contract = json!({
        "schema": "cairn.agent-completion.v2", "repository_id": "onixresearch/lattice",
        "registry_id": "lattice-release-mcp-observation-v1", "registry_hash": registry_hash,
        "selection_hash": selection_hash, "policy_hash": policy_hash,
        "final_subject_hash": subject_hash, "required_checks": checks,
    });
    let results = json!([{
        "check_id": "lattice-native-mcp-validation", "status": if passed { "passed" } else { "failed" },
        "command": command, "command_hash": command_hash, "subject_hash": subject_hash,
        "registry_hash": registry_hash, "selection_hash": selection_hash, "policy_hash": policy_hash,
        "evidence_hash": evidence_hash, "execution_receipt_hash": json_hash(observation),
        "artifacts": [
            { "path": "target/release-readiness/validation.json", "hash": evidence_hash },
            { "path": "target/release-readiness/validation.stderr", "hash": stderr_hash },
        ],
        "diagnostic": {
            "remediation_label": "inspect-native-mcp-validation", "summary": "Inspect the retained native MCP lifecycle validation observation; no heavyweight gate result is inferred.",
            "reproduction_argv": command, "working_root": "<repository-root>", "source": null,
        },
    }]);
    Ok(CompletionInputs {
        contract,
        results,
        passed,
    })
}
