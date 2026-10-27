use serde_json::Value;
use serde_json::json;

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
    if validation["isError"] != false || validation["structuredContent"]["mcp"]["read_only"] != true {
        return Err("native MCP validation envelope is absent or invalid".to_string());
    }
    let valid = validation["structuredContent"]["valid"]
        .as_bool()
        .ok_or("native MCP validation receipt lacks a boolean valid verdict")?;
    let exit_code = observation["exit_code"].as_i64().ok_or("native validation did not terminate with an exit code")?;
    let evidence_hash = observation["stdout_hash"].as_str().ok_or("native stdout identity is absent")?;
    let native_hash = observation["native_binary_hash"].as_str().ok_or("native executable identity is absent")?;
    for hash in [subject_hash, evidence_hash, native_hash] {
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
    let policy_hash = json_hash(policy);
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
        "artifacts": [{ "path": "target/release-readiness/validation.json", "hash": evidence_hash }],
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
