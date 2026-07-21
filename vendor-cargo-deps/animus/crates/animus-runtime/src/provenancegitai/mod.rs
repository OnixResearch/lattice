const PROVENANCE_GIT_AI_DISABLED_STATUS: &str = "provenance-disabled";
const PROVENANCE_GIT_AI_SCHEMA_VERSION: &str = "authorship/3.0.0";
const PROVENANCE_GIT_AI_NOTES_REF: &str = "refs/notes/ai";
const PROVENANCE_GIT_AI_BACKEND_DISABLED: &str = "disabled";
const PROVENANCE_GIT_AI_BACKEND_OPTIONAL: &str = "optional";
const PROVENANCE_GIT_AI_BACKEND_REQUIRED: &str = "required";
const PROVENANCE_GIT_AI_RECEIPT_SCHEMA: &str = "animus.provenance.git-ai.receipt.v1";
const PROVENANCE_GIT_AI_POLICY_SCHEMA: &str = "animus.provenance.git-ai.policy.v1";
const PROVENANCE_GIT_AI_COMPAT_ADAPTER_LABEL: &str = "git-ai-v3";
const PROVENANCE_GIT_AI_DEFAULT_BACKEND_LABEL: &str = "vcs-provenance";
const MIN_LINE_NUMBER: u32 = 1;
const ZERO_LENGTH: u32 = 0;
const MAX_RANGE_CORE_LINES: u64 = 16_384;
const CHECKPOINT_OBSERVATION_COUNT: u64 = 2;
const EXTERNAL_COMPAT_HASH_HEX_LENGTH: usize = 64;
const RANGE_SEPARATOR: char = '-';
const PATH_SEPARATOR: char = '/';
const CURRENT_DIRECTORY_SEGMENT: &str = ".";
const PARENT_DIRECTORY_SEGMENT: &str = "..";
const GIT_TERM_NEEDLES: &[&str] = &[
    "refs/notes/ai",
    "commit_sha",
    "commit sha",
    "branch",
    "stash",
    "rebase",
    "git_command",
    "git command",
    "git hook",
];
const CANONICAL_MUTATION_KINDS: &[&str] = &["edit", "write", "replace", "rename"];
const SAFE_RECEIPT_REDACTIONS: &[&str] = &[
    PUBLIC_REDACTION_MARKER,
    METADATA_ONLY_REDACTION_MARKER,
    REDACTED_PAYLOAD_MARKER,
];

include!("kindrange.rs");
include!("observation.rs");
include!("backendpolicy.rs");
include!("rangecore.rs");
include!("compat.rs");
