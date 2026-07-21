#[test]
fn safe_checkpoint_receipts_reject_leak_fixtures() {
    // r[verify agent_harness_runtime.provenance_git_ai.receipts]
    // r[verify agent_harness_runtime.provenance_git_ai.receipts.session_link]
    // r[verify agent_harness_runtime.provenance_git_ai.receipts.rejects_leaks]
    let checkpoint = checkpoint();
    let receipt = ProvenanceToolCheckpointReceipt::from_checkpoint(
        PROVENANCE_GIT_AI_RECEIPT_STATUS,
        GIT_AI_BACKEND_KIND,
        &checkpoint,
    )
    .expect("safe checkpoint receipt should validate");
    assert_eq!(receipt.session_id, GIT_AI_SESSION_ID);
    assert_eq!(receipt.trace_id, GIT_AI_TRACE_ID);
    assert_eq!(receipt.mutation_kind, ProvenanceMutationKind::Edit);
    assert!(!receipt.receipt_hash.contains(GIT_AI_RELATIVE_PATH));

    let unsafe_backend = ProvenanceToolCheckpointReceipt::from_checkpoint(
        PROVENANCE_GIT_AI_RECEIPT_STATUS,
        RAW_PROVIDER_FIXTURE,
        &checkpoint,
    )
    .expect_err("provider payload should be rejected");
    let leak_ranges = [line_range(FIRST_RANGE_START, FIRST_RANGE_END)];
    let unsafe_before = ProvenanceContentObservationDraft {
        file: ProvenanceFileIdentityDraft {
            relative_path: ABSOLUTE_PATH_FIXTURE,
            stable_id: GIT_AI_STABLE_ID,
            content_hash: BEFORE_CONTENT_HASH,
        },
        line_ranges: &leak_ranges,
        byte_count: OBSERVATION_BYTES,
    };
    let unsafe_after = observation(AFTER_CONTENT_HASH, AFTER_OBSERVATION_BYTES, &leak_ranges);
    let unsafe_path = ProvenanceToolCheckpoint::validate(&ProvenanceToolCheckpointDraft {
        session_id: GIT_AI_SESSION_ID,
        trace_id: GIT_AI_TRACE_ID,
        tool_name: GIT_AI_TOOL_NAME,
        mutation_kind: ProvenanceMutationKind::Edit.as_str(),
        before: unsafe_before,
        after: unsafe_after,
    })
    .expect_err("absolute path should be rejected");

    assert_eq!(unsafe_backend.kind(), ProvenanceErrorKind::UnsafeMetadata);
    assert_eq!(unsafe_path.kind(), ProvenanceErrorKind::UnsafeMetadata);
}

#[test]
fn verification_rail_has_git_optional_markers() {
    // r[verify agent_harness_runtime.provenance_git_ai.verification]
    let policy = tool_policy();
    assert_eq!(policy.backend_requirement, ProvenanceBackendRequirement::Required);
    assert_eq!(ProvenanceMutationKind::parse("rename").expect("rename should parse"), ProvenanceMutationKind::Rename);
}

fn valid_policy_draft<'a>() -> ProvenanceToolPolicyDraft<'a> {
    ProvenanceToolPolicyDraft {
        tool_name: GIT_AI_TOOL_NAME,
        mutation_kind: ProvenanceMutationKind::Edit.as_str(),
        file_identity_fields: FILE_IDENTITY_FIELDS,
        backend_requirement: ProvenanceBackendRequirement::Required.as_str(),
        receipt_redaction: METADATA_ONLY_REDACTION_MARKER,
    }
}
