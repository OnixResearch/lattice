const GIT_AI_RELATIVE_PATH: &str = "crates/animus-runtime/src/lib.rs";
const GIT_AI_STABLE_ID: &str = "file-stable-id";
const GIT_AI_TOOL_NAME: &str = "edit";
const GIT_AI_SESSION_ID: &str = "session-safe";
const GIT_AI_TRACE_ID: &str = "trace-safe";
const GIT_AI_BACKEND_KIND: &str = "memory-vcs";
const PROVENANCE_GIT_AI_RECEIPT_STATUS: &str = "provenance-recorded";
const BEFORE_CONTENT_HASH: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";
const AFTER_CONTENT_HASH: &str =
    "2222222222222222222222222222222222222222222222222222222222222222";
const FILE_IDENTITY_FIELDS: &[&str] = &["relative_path", "stable_id", "content_hash"];
const FILE_IDENTITY_FIELD_COUNT: usize = 3;
const FIRST_RANGE_START: u32 = 2;
const FIRST_RANGE_END: u32 = 4;
const SECOND_RANGE_START: u32 = 6;
const SECOND_RANGE_END: u32 = 7;
const COMPRESSED_RANGE_START: u32 = 2;
const COMPRESSED_RANGE_END: u32 = 7;
const DELETE_HUNK_OLD_START: u32 = 3;
const DELETE_HUNK_OLD_LEN: u32 = 1;
const DELETE_HUNK_NEW_START: u32 = 3;
const INSERT_HUNK_OLD_START: u32 = 6;
const INSERT_HUNK_NEW_START: u32 = 5;
const INSERT_HUNK_NEW_LEN: u32 = 2;
const EXPECTED_ACCEPTED_LINES: u64 = 4;
const EXPECTED_SHIFTED_RANGE_START: u32 = 2;
const EXPECTED_SHIFTED_RANGE_END: u32 = 3;
const EXPECTED_SHIFTED_SECOND_START: u32 = 7;
const EXPECTED_SHIFTED_SECOND_END: u32 = 8;
const OBSERVATION_BYTES: u64 = 128;
const AFTER_OBSERVATION_BYTES: u64 = 144;
const EXTERNAL_COMPAT_ID: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const EXTERNAL_COMPAT_ID_ALT: &str =
    "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
const GIT_BRANCH_FIELD: &str = "branch";
const GIT_FIELD_POLICY_FIELDS: &[&str] = &["relative_path", GIT_BRANCH_FIELD];
const ABSOLUTE_PATH_FIXTURE: &str = "/home/operator/worktree/src/lib.rs";
const RAW_PROVIDER_FIXTURE: &str = "provider_json { secret }";
const UNSUPPORTED_MUTATION_KIND: &str = "git-commit";
const UNKNOWN_BACKEND_REQUIREMENT: &str = "git-only";
const UNSAFE_REDACTION_MARKER: &str = "raw_prompt";
const MALFORMED_COMPAT_ID: &str = "not-a-hex-key";

#[derive(Default)]
struct MemoryBackend {
    checkpoints: Collection<ProvenanceToolCheckpoint>,
}

impl VersionControlProvenanceBackend for MemoryBackend {
    fn backend_kind(&self) -> &str {
        GIT_AI_BACKEND_KIND
    }

    fn record_checkpoint(
        &mut self,
        checkpoint: &ProvenanceToolCheckpoint,
    ) -> Result<ProvenanceToolCheckpointReceipt, ProvenanceError> {
        self.checkpoints.push(checkpoint.clone());
        ProvenanceToolCheckpointReceipt::from_checkpoint(
            PROVENANCE_GIT_AI_RECEIPT_STATUS,
            self.backend_kind(),
            checkpoint,
        )
    }
}

fn line_range(start: u32, end: u32) -> ProvenanceLineRange {
    ProvenanceLineRange::new(start, end).expect("line range should validate")
}

fn file_identity(content_hash: &str) -> ProvenanceFileIdentityDraft<'_> {
    ProvenanceFileIdentityDraft {
        relative_path: GIT_AI_RELATIVE_PATH,
        stable_id: GIT_AI_STABLE_ID,
        content_hash,
    }
}

fn observation<'a>(
    content_hash: &'a str,
    byte_count: u64,
    line_ranges: &'a [ProvenanceLineRange],
) -> ProvenanceContentObservationDraft<'a> {
    ProvenanceContentObservationDraft {
        file: file_identity(content_hash),
        line_ranges,
        byte_count,
    }
}

fn checkpoint() -> ProvenanceToolCheckpoint {
    let ranges = [
        line_range(FIRST_RANGE_START, FIRST_RANGE_END),
        line_range(SECOND_RANGE_START, SECOND_RANGE_END),
    ];
    ProvenanceToolCheckpoint::validate(&ProvenanceToolCheckpointDraft {
        session_id: GIT_AI_SESSION_ID,
        trace_id: GIT_AI_TRACE_ID,
        tool_name: GIT_AI_TOOL_NAME,
        mutation_kind: ProvenanceMutationKind::Edit.as_str(),
        before: observation(BEFORE_CONTENT_HASH, OBSERVATION_BYTES, &ranges),
        after: observation(AFTER_CONTENT_HASH, AFTER_OBSERVATION_BYTES, &ranges),
    })
    .expect("checkpoint should validate")
}

fn tool_policy() -> ProvenanceToolPolicy {
    ProvenanceToolPolicy::validate(&ProvenanceToolPolicyDraft {
        tool_name: GIT_AI_TOOL_NAME,
        mutation_kind: ProvenanceMutationKind::Edit.as_str(),
        file_identity_fields: FILE_IDENTITY_FIELDS,
        backend_requirement: ProvenanceBackendRequirement::Required.as_str(),
        receipt_redaction: METADATA_ONLY_REDACTION_MARKER,
    })
    .expect("policy should validate")
}
