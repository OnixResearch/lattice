use super::*;

#[test]
fn attempt_result_hashes_outputs_and_round_trips() {
    let receipt = crate::candidate::CandidateReceipt {
        schema: crate::candidate::CANDIDATE_RECEIPT_SCHEMA.to_string(),
        work_id: "work-1".to_string(),
        attempt_id: "attempt-1".to_string(),
        fence: "7".to_string(),
        workspace_id: "workspace-1".to_string(),
        base_identity: "a".repeat(64),
        candidate_identity: Some("b".repeat(64)),
        diff_identity: Some("c".repeat(64)),
        changed_paths: vec!["src/main.rs".to_string()],
        disposition: crate::candidate::CandidateDisposition::Produced,
        blockers: Vec::new(),
    };
    let outcome = Outcome::Produced {
        receipt: Box::new(receipt.clone()),
        changed_files: std::collections::BTreeMap::new(),
        stdout: b"stdout".to_vec(),
        stderr: b"stderr".to_vec(),
    };
    let result = AttemptResult::from_outcome(&outcome);
    assert_eq!(result.stdout_blake3, crate::pure_inet_profile::blake3_identity(b"stdout"));
    assert_eq!(result.stderr_blake3, crate::pure_inet_profile::blake3_identity(b"stderr"));
    let json = result.to_json().expect("result serializes");
    let parsed = AttemptResult::from_json(&json).expect("result parses");
    assert_eq!(parsed, result);

    let mut invalid = AttemptResult::from_outcome(&Outcome::Blocked {
        blocker: "blocked".to_string(),
    });
    invalid.receipt = Some(receipt);
    assert!(invalid.validate().is_err());
}

#[test]
fn cancelled_and_unknown_results_are_valid_non_success() {
    let cancelled = AttemptResult::from_outcome(&Outcome::Cancelled {
        stdout: Vec::new(),
        stderr: Vec::new(),
    });
    assert_eq!(cancelled.disposition, crate::candidate::CandidateDisposition::Cancelled);
    assert!(cancelled.receipt.is_none());
    assert!(cancelled.published_artifacts.is_empty());
    assert!(cancelled.validate().is_ok());

    let unknown = AttemptResult::from_outcome(&Outcome::Unknown {
        blocker: "capture interrupted".to_string(),
        stdout: b"partial".to_vec(),
        stderr: Vec::new(),
    });
    assert_eq!(unknown.disposition, crate::candidate::CandidateDisposition::Unknown);
    assert!(unknown.blocker.is_some());
    assert!(unknown.validate().is_ok());
}

#[test]
fn attempt_result_rejects_invalid_artifact_paths() {
    let receipt = crate::candidate::CandidateReceipt {
        schema: crate::candidate::CANDIDATE_RECEIPT_SCHEMA.to_string(),
        work_id: "work-1".to_string(),
        attempt_id: "attempt-1".to_string(),
        fence: "7".to_string(),
        workspace_id: "workspace-1".to_string(),
        base_identity: "a".repeat(64),
        candidate_identity: Some("b".repeat(64)),
        diff_identity: Some("c".repeat(64)),
        changed_paths: vec!["src/main.rs".to_string()],
        disposition: crate::candidate::CandidateDisposition::Produced,
        blockers: Vec::new(),
    };
    let mut result = AttemptResult::from_outcome(&Outcome::Produced {
        receipt: Box::new(receipt),
        changed_files: std::collections::BTreeMap::from([("src/main.rs".to_string(), b"main".to_vec())]),
        stdout: Vec::new(),
        stderr: Vec::new(),
    });

    result.published_artifacts = vec!["src/other.rs".to_string()];
    assert!(result.validate().is_err());

    result.published_artifacts = vec!["src/main.rs".to_string(), "src/main.rs".to_string()];
    assert!(result.validate().is_err());

    let mut blocked = AttemptResult::from_outcome(&Outcome::Blocked {
        blocker: "blocked".to_string(),
    });
    blocked.published_artifacts = vec!["src/main.rs".to_string()];
    assert!(blocked.validate().is_err());
}
