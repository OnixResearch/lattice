#[test]
fn architecture_scope_rejects_git_terms_from_core_fields() {
    // r[verify agent_harness_runtime.provenance_git_ai.scope]
    // r[verify agent_harness_runtime.provenance_git_ai.domain.rejects_git_terms]
    validate_vcs_neutral_field_names(&[
        "relative_path",
        "stable_id",
        "content_hash",
        "line_ranges",
        "session_id",
        "trace_id",
        "mutation_kind",
    ])
    .expect("neutral fields should validate");

    let error = validate_vcs_neutral_field_names(&[GIT_BRANCH_FIELD])
        .expect_err("Git-specific field should be rejected");
    assert_eq!(error.kind(), ProvenanceErrorKind::GitTermInCoreDto);
}

#[test]
fn provenance_domain_validates_observations_without_vcs() {
    // r[verify agent_harness_runtime.provenance_git_ai.domain]
    // r[verify agent_harness_runtime.provenance_git_ai.domain.no_vcs]
    let checkpoint = checkpoint();

    assert_eq!(checkpoint.session_id, GIT_AI_SESSION_ID);
    assert_eq!(checkpoint.trace_id, GIT_AI_TRACE_ID);
    assert_eq!(checkpoint.mutation_kind, ProvenanceMutationKind::Edit);
    assert_eq!(checkpoint.before.file.relative_path, GIT_AI_RELATIVE_PATH);
    assert_ne!(checkpoint.before.file.content_hash, checkpoint.after.file.content_hash);
}

#[test]
fn in_memory_backend_records_checkpoints_without_git() {
    // r[verify agent_harness_runtime.provenance_git_ai.vcs_backend]
    // r[verify agent_harness_runtime.provenance_git_ai.vcs_backend.in_memory]
    // r[verify agent_harness_runtime.provenance_git_ai.tool_policy.valid]
    let checkpoint = checkpoint();
    let mut backend = MemoryBackend::default();
    let receipt = record_provenance_checkpoint(
        ProvenanceBackendRequirement::Required,
        Some(&mut backend),
        &checkpoint,
    )
    .expect("required backend should record checkpoint");

    assert_eq!(backend.checkpoints.len(), 1);
    assert_eq!(backend.checkpoints[0], checkpoint);
    assert_eq!(receipt.status, PROVENANCE_GIT_AI_RECEIPT_STATUS);
    assert_eq!(receipt.backend_kind, GIT_AI_BACKEND_KIND);
    assert_eq!(receipt.observation_count, CHECKPOINT_OBSERVATION_COUNT);
    assert!(!receipt.receipt_hash.contains(GIT_AI_RELATIVE_PATH));
}

#[test]
fn missing_required_backend_fails_before_evidence_claim() {
    // r[verify agent_harness_runtime.provenance_git_ai.vcs_backend.missing]
    let checkpoint = checkpoint();
    let error = record_provenance_checkpoint(
        ProvenanceBackendRequirement::Required,
        None,
        &checkpoint,
    )
    .expect_err("missing required backend should fail closed");

    assert_eq!(error.kind(), ProvenanceErrorKind::UnavailableBackend);
    assert_eq!(error.field(), "provenance_backend");
}

#[test]
fn nickel_governed_policy_classifies_mutating_tools() {
    // r[verify agent_harness_runtime.provenance_git_ai.tool_policy]
    // r[verify agent_harness_runtime.provenance_git_ai.tool_policy.valid]
    let policy = tool_policy();
    let policies = [policy.clone()];
    let selected = classify_provenance_tool_policy(&policies, GIT_AI_TOOL_NAME)
        .expect("tool policy should classify edit tool");

    assert_eq!(selected.tool_name, GIT_AI_TOOL_NAME);
    assert_eq!(selected.mutation_kind, ProvenanceMutationKind::Edit);
    assert_eq!(selected.backend_requirement, ProvenanceBackendRequirement::Required);
    assert_eq!(selected.receipt_redaction, METADATA_ONLY_REDACTION_MARKER);
    assert_eq!(selected.file_identity_fields.len(), FILE_IDENTITY_FIELD_COUNT);
    assert!(!selected.policy_hash.contains(GIT_AI_TOOL_NAME));
}

#[test]
fn invalid_policy_fixtures_fail_closed() {
    // r[verify agent_harness_runtime.provenance_git_ai.tool_policy.invalid]
    let unknown_kind = ProvenanceToolPolicy::validate(&ProvenanceToolPolicyDraft {
        mutation_kind: UNSUPPORTED_MUTATION_KIND,
        ..valid_policy_draft()
    })
    .expect_err("unknown mutation kind should fail");
    let unknown_backend = ProvenanceToolPolicy::validate(&ProvenanceToolPolicyDraft {
        backend_requirement: UNKNOWN_BACKEND_REQUIREMENT,
        ..valid_policy_draft()
    })
    .expect_err("unknown backend requirement should fail");
    let unsafe_redaction = ProvenanceToolPolicy::validate(&ProvenanceToolPolicyDraft {
        receipt_redaction: UNSAFE_REDACTION_MARKER,
        ..valid_policy_draft()
    })
    .expect_err("unsafe redaction should fail");
    let git_field = ProvenanceToolPolicy::validate(&ProvenanceToolPolicyDraft {
        file_identity_fields: GIT_FIELD_POLICY_FIELDS,
        ..valid_policy_draft()
    })
    .expect_err("Git field should fail");

    assert_eq!(unknown_kind.kind(), ProvenanceErrorKind::InvalidPolicy);
    assert_eq!(unknown_backend.kind(), ProvenanceErrorKind::InvalidPolicy);
    assert_eq!(unsafe_redaction.kind(), ProvenanceErrorKind::InvalidPolicy);
    assert_eq!(git_field.kind(), ProvenanceErrorKind::GitTermInCoreDto);
}
