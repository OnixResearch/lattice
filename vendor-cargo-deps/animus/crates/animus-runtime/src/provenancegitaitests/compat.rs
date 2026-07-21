#[test]
fn git_ai_v3_compatibility_round_trips_at_adapter_boundary() {
    // r[verify agent_harness_runtime.provenance_git_ai.git_ai_adapter]
    // r[verify agent_harness_runtime.provenance_git_ai.git_ai_adapter.round_trip]
    let log = ProvenanceGitAiAuthorshipLog::validate(&ProvenanceGitAiAuthorshipLogDraft {
        schema_version: PROVENANCE_GIT_AI_SCHEMA_VERSION,
        storage_ref: PROVENANCE_GIT_AI_NOTES_REF,
        session_key: EXTERNAL_COMPAT_ID,
        trace_key: EXTERNAL_COMPAT_ID_ALT,
        known_human_key: EXTERNAL_COMPAT_ID,
        legacy_prompt_key: EXTERNAL_COMPAT_ID_ALT,
        file_attestations: &[ProvenanceGitAiFileAttestationDraft {
            relative_path: GIT_AI_RELATIVE_PATH,
            line_ranges: &[line_range(FIRST_RANGE_START, FIRST_RANGE_END)],
            external_file_key: EXTERNAL_COMPAT_ID,
        }],
        metadata: &[ProvenanceGitAiMetadataDraft {
            key: "model",
            value: MODEL_LABEL,
        }],
    })
    .expect("Git AI v3 compatibility log should validate");

    assert_eq!(ProvenanceGitAiAuthorshipLog::adapter_label(), "git-ai-v3");
    assert_eq!(log.schema_version, PROVENANCE_GIT_AI_SCHEMA_VERSION);
    assert_eq!(log.storage_ref, PROVENANCE_GIT_AI_NOTES_REF);
    assert_eq!(log.file_attestations[0].line_ranges, [line_range(FIRST_RANGE_START, FIRST_RANGE_END)]);
    assert_eq!(log.metadata[0].key, "model");
}

#[test]
fn git_ai_adapter_rejects_malformed_compatibility_data() {
    // r[verify agent_harness_runtime.provenance_git_ai.git_ai_adapter]
    let error = ProvenanceGitAiAuthorshipLog::validate(&ProvenanceGitAiAuthorshipLogDraft {
        schema_version: PROVENANCE_GIT_AI_SCHEMA_VERSION,
        storage_ref: PROVENANCE_GIT_AI_NOTES_REF,
        session_key: MALFORMED_COMPAT_ID,
        trace_key: EXTERNAL_COMPAT_ID_ALT,
        known_human_key: EXTERNAL_COMPAT_ID,
        legacy_prompt_key: EXTERNAL_COMPAT_ID_ALT,
        file_attestations: &[],
        metadata: &[],
    })
    .expect_err("malformed external id should fail");

    assert_eq!(error.kind(), ProvenanceErrorKind::InvalidCompatibility);
}

#[test]
fn non_git_backend_does_not_require_git_ai_ids_or_notes() {
    // r[verify agent_harness_runtime.provenance_git_ai.git_ai_adapter.not_core]
    // r[verify agent_harness_runtime.provenance_git_ai.verification.git_optional]
    let checkpoint = checkpoint();
    let mut backend = MemoryBackend::default();
    let receipt = record_provenance_checkpoint(
        ProvenanceBackendRequirement::Required,
        Some(&mut backend),
        &checkpoint,
    )
    .expect("non-Git backend should record checkpoint");

    assert_eq!(receipt.backend_kind, GIT_AI_BACKEND_KIND);
    assert!(!receipt.receipt_hash.contains(PROVENANCE_GIT_AI_NOTES_REF));
    assert!(!receipt.receipt_hash.contains(EXTERNAL_COMPAT_ID));
}
