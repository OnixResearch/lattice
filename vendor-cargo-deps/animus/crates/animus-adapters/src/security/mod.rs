//! Data-only security review contracts adapted from Goose layered review patterns.

mod decision;
mod evidence;
mod input;
mod pattern;
mod signal;
mod types;

pub use decision::Binding;
pub use decision::FailBehavior;
pub use decision::FailPolicy;
pub use decision::Gate;
pub use decision::Issue;
pub use decision::Outcome;
pub use decision::Snapshot;
pub use decision::Status;
pub use decision::bind;
pub use evidence::Leak;
pub use evidence::Receipt;
pub use evidence::validate_text;
pub use input::Draft;
pub use input::Review;
pub use input::ToolCallDescriptor;
pub use input::ToolCallDescriptorDraft;
pub use pattern::scan_review_input;
pub use signal::Category;
pub use signal::Confidence;
pub use signal::Disposition;
pub use signal::Finding;
pub use signal::Report;
pub use signal::ScannerStatus;
pub use types::ActionClass;
pub use types::AdvisoryOutcome;
pub use types::AdvisoryReview;
pub use types::AdvisoryReviewDraft;
pub use types::Error;
pub use types::RiskClass;

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

/// Character length of lowercase BLAKE3 hex evidence identities.
pub const SECURITY_REVIEW_HASH_HEX_CHARS: usize = crate::artifact::BLAKE3_HASH_HEX_CHARS;

const HASH_SCHEMA: &str = "animus.goose-security-review.v1";
const HASH_SEPARATOR: &str = "|";
const SEARCH_SEPARATOR: char = ' ';
const FIRST_EVIDENCE_INDEX: usize = 0;
const MAX_TASK_LABEL_ITEMS: usize = 16;
const MAX_CONTEXT_SUMMARY_ITEMS: usize = 16;
const MAX_ARGUMENT_LABEL_ITEMS: usize = 16;
const MAX_POLICY_LABEL_ITEMS: usize = 16;
const MAX_EVIDENCE_ITEMS: usize = 16;
const MAX_ADVISORY_EVIDENCE_ITEMS: usize = 16;
const RECEIPT_COUNT_LIMIT: u32 = u32::MAX;

const TASK_LABEL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("security_review_task_label");
const CONTEXT_SUMMARY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("security_review_context_summary");
const TOOL_NAME_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("security_review_tool_name");
const TOOL_TARGET_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("security_review_tool_target_label");
const ARGUMENT_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("security_review_argument_label");
const POLICY_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("security_review_policy_label");
const RAW_EVIDENCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("security_review_raw_evidence");
const REVIEWER_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("security_review_reviewer_label");
const REVIEW_EXPLANATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("security_review_explanation_label");

const RAW_PROMPT_MARKERS: &[&str] = &["RAW_PROMPT:", "<raw-prompt>", "raw prompt"];
const CREDENTIAL_MARKERS: &[&str] = &[
    "BEGIN OPENSSH PRIVATE KEY",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "AWS_SECRET_ACCESS_KEY",
    "sk-",
];
const PRIVATE_PATH_MARKERS: &[&str] = &["/home/", "C:\\Users\\", "\\Users\\"];
const PROVIDER_PAYLOAD_MARKERS: &[&str] = &["PROVIDER_PAYLOAD:", "raw_provider_payload"];
const RAW_TOOL_ARGUMENT_MARKERS: &[&str] = &["RAW_TOOL_ARGUMENT:", "raw_tool_argument"];
const RAW_TOOL_OUTPUT_MARKERS: &[&str] = &["UNBOUNDED_OUTPUT:", "raw_tool_output"];

const PROMPT_INJECTION_PATTERNS: &[&str] = &[
    "ignore previous",
    "hidden instruction",
    "system prompt",
    "developer message",
    "prompt injection",
];
const DESTRUCTIVE_ACTION_PATTERNS: &[&str] = &["recursive-delete", "rm -rf", "delete workspace", "destructive-action"];
const EXFILTRATION_PATTERNS: &[&str] = &[
    "unknown-upload-domain",
    "exfiltrate",
    "ssh-key",
    "credential-access",
    "upload secret",
];
const PRIVILEGE_ESCALATION_PATTERNS: &[&str] = &["sudo", "root-required", "privilege-escalation", "chmod 777"];
const OBFUSCATED_REMOTE_EXECUTION_PATTERNS: &[&str] = &[
    "download-execute",
    "curl-pipe-sh",
    "curl | sh",
    "base64-decode-exec",
    "powershell encodedcommand",
];
const OUT_OF_SCOPE_MUTATION_PATTERNS: &[&str] = &[
    "out-of-scope",
    "outside-workspace",
    "unauthorized workspace",
    "cross-project mutation",
];

const _: () = assert!(SECURITY_REVIEW_HASH_HEX_CHARS > 0);
const _: () = assert!(MAX_TASK_LABEL_ITEMS > 0);
const _: () = assert!(MAX_CONTEXT_SUMMARY_ITEMS > 0);
const _: () = assert!(MAX_ARGUMENT_LABEL_ITEMS > 0);
const _: () = assert!(MAX_POLICY_LABEL_ITEMS > 0);
const _: () = assert!(MAX_EVIDENCE_ITEMS > 0);
const _: () = assert!(MAX_ADVISORY_EVIDENCE_ITEMS > 0);

fn hash_parts(parts: &[&str]) -> Text {
    let mut text = Text::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            text.push_str(HASH_SEPARATOR);
        }
        text.push_str(part);
    }
    alloc::format!("{}", blake3::hash(text.as_bytes()).to_hex())
}

fn receipt_count(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(RECEIPT_COUNT_LIMIT)
}
