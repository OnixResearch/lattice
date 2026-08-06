//! Host tool executor capability-pack contracts.
//!
//! Reusable Animus code describes and admits host-owned executors. Concrete
//! plugins, shell commands, filesystem/process/network authority, and product
//! catalog discovery remain outside this pure module.

// Cairn adaptation compatibility module keeps DTOs, validators, and fixtures compact while the change is drained.
#![allow(missing_docs)]
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports,
    path_segment_repetition,
    underscore_in_module_filename,
    dead_code
)]
#![allow(numeric_units, raw_arithmetic_overflow, unbounded_collection_growth)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools
)]

const HASH_DOMAIN: &str = "animus.tool-executor-packs.v1";
const HASH_SEPARATOR: &str = "|";
const RECEIPT_SCHEMA: &str = "animus.tool-executor-pack.receipt.v1";
const MAX_LABEL_CHARS: usize = 96;
const HASH_HEX_CHARS: usize = 64;
const EMPTY_COUNT: u32 = 0;
const MIN_COUNT: u32 = 1;
const MAX_OUTPUT_BYTES: u32 = 4_096;
const UNSAFE_MARKERS: &[&str] = &[
    "RAW_TOOL_OUTPUT:",
    "RAW_PROMPT:",
    "PROVIDER_PAYLOAD:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "sk-",
    "/home/",
    "http://",
    "https://",
    "UNBOUNDED_OUTPUT:",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolMutationPolicy {
    ReadOnly,
    BoundedMutation,
    MutationDenied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolApprovalState {
    Approved,
    Denied,
    Required,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutorReadiness {
    Ready,
    Missing,
    Unavailable,
    TimedOut,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptExposureState {
    Hidden,
    Exposed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolDefinitionMetadata {
    pub tool_name: String,
    pub schema_hash: String,
    pub trusted: bool,
    pub active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutorPack {
    pub pack_label: String,
    pub tool_name: String,
    pub executor_label: String,
    pub required_capability: String,
    pub output_policy: ToolOutputPolicy,
    pub mutation_policy: ToolMutationPolicy,
    pub approval_state: ToolApprovalState,
    pub prompt_exposure: PromptExposureState,
    pub readiness: ExecutorReadiness,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolOutputPolicy {
    pub max_output_bytes: u32,
    pub hash_only: bool,
}

impl Default for ToolOutputPolicy {
    fn default() -> Self {
        Self {
            max_output_bytes: MAX_OUTPUT_BYTES,
            hash_only: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolDispatchRequest {
    pub definition: ToolDefinitionMetadata,
    pub pack: Option<ToolExecutorPack>,
    pub requested_capability: String,
    pub argument_hash: String,
    pub mutating: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FakeExecutorOutput {
    pub output_summary: String,
    pub output_bytes: u32,
    pub malformed: bool,
    pub unsafe_reference: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutorReceipt {
    pub schema: String,
    pub tool_name: String,
    pub executor_label: String,
    pub status: ExecutorOutcome,
    pub argument_hash: String,
    pub output_hash: String,
    pub redacted: bool,
    pub issue_codes: Vec<String>,
    pub receipt_hash: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutorOutcome {
    Completed,
    Denied,
    Missing,
    Unavailable,
    TimedOut,
    Cancelled,
    MalformedOutput,
    UnsafeReference,
    UnboundedOutput,
    MutationConflict,
    LeakRejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutorIssue {
    EmptyField,
    InvalidHash,
    MissingExecutor,
    CapabilityGap,
    DeniedApproval,
    MalformedArguments,
    MalformedOutput,
    UnsafeReference,
    UnboundedOutput,
    MutationConflict,
    TimedOut,
    Cancelled,
    Unavailable,
    ReceiptLeak,
    UntrustedDefinition,
}

impl ExecutorIssue {
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptyField => "empty-field",
            Self::InvalidHash => "invalid-hash",
            Self::MissingExecutor => "missing-executor",
            Self::CapabilityGap => "capability-gap",
            Self::DeniedApproval => "denied-approval",
            Self::MalformedArguments => "malformed-arguments",
            Self::MalformedOutput => "malformed-output",
            Self::UnsafeReference => "unsafe-reference",
            Self::UnboundedOutput => "unbounded-output",
            Self::MutationConflict => "mutation-conflict",
            Self::TimedOut => "timed-out",
            Self::Cancelled => "cancelled",
            Self::Unavailable => "unavailable",
            Self::ReceiptLeak => "receipt-leak",
            Self::UntrustedDefinition => "untrusted-definition",
        }
    }
}

pub type ExecutorResult<T> = Result<T, Vec<ExecutorIssue>>;

pub fn validate_executor_pack(pack: &ToolExecutorPack) -> ExecutorResult<()> {
    let mut issues = Vec::new();
    validate_label(pack.pack_label.as_str(), &mut issues);
    validate_label(pack.tool_name.as_str(), &mut issues);
    validate_label(pack.executor_label.as_str(), &mut issues);
    validate_label(pack.required_capability.as_str(), &mut issues);
    if pack.output_policy.max_output_bytes < MIN_COUNT {
        issues.push(ExecutorIssue::UnboundedOutput);
    }
    result_from_issues(issues, ())
}

pub fn admit_tool_dispatch(request: &ToolDispatchRequest) -> ExecutorResult<ToolExecutorPack> {
    let mut issues = Vec::new();
    validate_definition(&request.definition, &mut issues);
    validate_label(request.requested_capability.as_str(), &mut issues);
    validate_hash(request.argument_hash.as_str(), &mut issues);
    let Some(pack) = &request.pack else {
        issues.push(ExecutorIssue::MissingExecutor);
        return result_from_issues(issues, ToolExecutorPack::placeholder());
    };
    append_result(validate_executor_pack(pack), &mut issues);
    if pack.tool_name != request.definition.tool_name {
        issues.push(ExecutorIssue::MissingExecutor);
    }
    if pack.required_capability != request.requested_capability {
        issues.push(ExecutorIssue::CapabilityGap);
    }
    if pack.approval_state != ToolApprovalState::Approved {
        issues.push(ExecutorIssue::DeniedApproval);
    }
    if request.mutating && pack.mutation_policy != ToolMutationPolicy::BoundedMutation {
        issues.push(ExecutorIssue::MutationConflict);
    }
    match pack.readiness {
        ExecutorReadiness::Ready => {}
        ExecutorReadiness::Missing => issues.push(ExecutorIssue::MissingExecutor),
        ExecutorReadiness::Unavailable => issues.push(ExecutorIssue::Unavailable),
        ExecutorReadiness::TimedOut => issues.push(ExecutorIssue::TimedOut),
        ExecutorReadiness::Cancelled => issues.push(ExecutorIssue::Cancelled),
    }
    result_from_issues(issues, pack.clone())
}

pub fn run_fake_executor(
    request: &ToolDispatchRequest,
    output: &FakeExecutorOutput,
) -> ExecutorResult<ExecutorReceipt> {
    let pack = admit_tool_dispatch(request)?;
    let mut issues = Vec::new();
    validate_label(output.output_summary.as_str(), &mut issues);
    if output.malformed {
        issues.push(ExecutorIssue::MalformedOutput);
    }
    if output.output_bytes > pack.output_policy.max_output_bytes {
        issues.push(ExecutorIssue::UnboundedOutput);
    }
    if let Some(reference) = &output.unsafe_reference {
        if UNSAFE_MARKERS.iter().any(|marker| reference.contains(marker)) {
            issues.push(ExecutorIssue::UnsafeReference);
        }
    }
    if !issues.is_empty() {
        return Err(sorted_issues(issues));
    }
    let output_hash = tool_executor_hash("output", output.output_summary.as_bytes());
    let mut receipt = ExecutorReceipt {
        schema: RECEIPT_SCHEMA.to_owned(),
        tool_name: request.definition.tool_name.clone(),
        executor_label: pack.executor_label,
        status: ExecutorOutcome::Completed,
        argument_hash: request.argument_hash.clone(),
        output_hash,
        redacted: true,
        issue_codes: Vec::new(),
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = receipt_hash(&receipt);
    validate_executor_receipt(&receipt)?;
    Ok(receipt)
}

pub fn validate_executor_receipt(receipt: &ExecutorReceipt) -> ExecutorResult<()> {
    let mut issues = Vec::new();
    if receipt.schema != RECEIPT_SCHEMA {
        issues.push(ExecutorIssue::EmptyField);
    }
    validate_label(receipt.tool_name.as_str(), &mut issues);
    validate_label(receipt.executor_label.as_str(), &mut issues);
    validate_hash(receipt.argument_hash.as_str(), &mut issues);
    validate_hash(receipt.output_hash.as_str(), &mut issues);
    validate_hash(receipt.receipt_hash.as_str(), &mut issues);
    if !receipt.redacted {
        issues.push(ExecutorIssue::ReceiptLeak);
    }
    for code in &receipt.issue_codes {
        validate_label(code.as_str(), &mut issues);
    }
    result_from_issues(issues, ())
}

pub fn tool_executor_hash(label: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(HASH_DOMAIN.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

impl ToolExecutorPack {
    fn placeholder() -> Self {
        Self {
            pack_label: String::new(),
            tool_name: String::new(),
            executor_label: String::new(),
            required_capability: String::new(),
            output_policy: ToolOutputPolicy::default(),
            mutation_policy: ToolMutationPolicy::ReadOnly,
            approval_state: ToolApprovalState::Denied,
            prompt_exposure: PromptExposureState::Hidden,
            readiness: ExecutorReadiness::Missing,
        }
    }
}

fn validate_definition(definition: &ToolDefinitionMetadata, issues: &mut Vec<ExecutorIssue>) {
    validate_label(definition.tool_name.as_str(), issues);
    validate_hash(definition.schema_hash.as_str(), issues);
    if !definition.trusted || !definition.active {
        issues.push(ExecutorIssue::UntrustedDefinition);
    }
}

fn validate_label(value: &str, issues: &mut Vec<ExecutorIssue>) {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_LABEL_CHARS {
        issues.push(ExecutorIssue::EmptyField);
    }
    if UNSAFE_MARKERS.iter().any(|marker| trimmed.contains(marker)) {
        issues.push(ExecutorIssue::ReceiptLeak);
    }
}

fn validate_hash(value: &str, issues: &mut Vec<ExecutorIssue>) {
    if value.len() != HASH_HEX_CHARS || !value.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        issues.push(ExecutorIssue::InvalidHash);
    }
}

fn append_result(result: ExecutorResult<()>, issues: &mut Vec<ExecutorIssue>) {
    if let Err(mut nested) = result {
        issues.append(&mut nested);
    }
}

fn result_from_issues<T>(issues: Vec<ExecutorIssue>, value: T) -> ExecutorResult<T> {
    let issues = sorted_issues(issues);
    if issues.is_empty() { Ok(value) } else { Err(issues) }
}

fn sorted_issues(mut issues: Vec<ExecutorIssue>) -> Vec<ExecutorIssue> {
    issues.sort_by_key(|issue| issue.code());
    issues.dedup();
    issues
}

fn receipt_hash(receipt: &ExecutorReceipt) -> String {
    let canonical = format!(
        "{}{}{}{}{}{}{}{}{}",
        receipt.schema,
        HASH_SEPARATOR,
        receipt.tool_name,
        HASH_SEPARATOR,
        receipt.executor_label,
        HASH_SEPARATOR,
        receipt.argument_hash,
        HASH_SEPARATOR,
        receipt.output_hash
    );
    tool_executor_hash("receipt", canonical.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOOL_NAME: &str = "read_file_summary";
    const PACK_LABEL: &str = "readonly-pack";
    const EXECUTOR_LABEL: &str = "fake-readonly-executor";
    const CAPABILITY: &str = "tool.readonly.summary";
    const SCHEMA_BYTES: &[u8] = b"schema";
    const ARGUMENT_BYTES: &[u8] = b"args";
    const OUTPUT_SUMMARY: &str = "bounded summary";
    const OUTPUT_BYTES: u32 = 15;

    #[test]
    fn read_only_pack_admits_and_runs_fake_executor() {
        // r[verify agent_harness_runtime.tool_executor_packs.descriptor]
        // r[verify agent_harness_runtime.tool_executor_packs.admission]
        // r[verify agent_harness_runtime.tool_executor_packs.fake_executors]
        let request = request(Some(pack(ToolApprovalState::Approved, ExecutorReadiness::Ready)));
        let output = FakeExecutorOutput {
            output_summary: OUTPUT_SUMMARY.to_owned(),
            output_bytes: OUTPUT_BYTES,
            malformed: false,
            unsafe_reference: None,
        };

        let receipt = run_fake_executor(&request, &output).expect("fake executor");

        assert_eq!(receipt.status, ExecutorOutcome::Completed);
        assert_eq!(receipt.tool_name, TOOL_NAME);
        assert_eq!(receipt.executor_label, EXECUTOR_LABEL);
        assert!(validate_executor_receipt(&receipt).is_ok());
    }

    #[test]
    fn dispatch_fails_closed_for_missing_capability_denial_and_mutation_conflict() {
        // r[verify agent_harness_runtime.tool_executor_packs.receipts]
        // r[verify agent_harness_runtime.tool_executor_packs.verification]
        let mut missing = request(None);
        assert!(admit_tool_dispatch(&missing).expect_err("missing").contains(&ExecutorIssue::MissingExecutor));

        missing.pack = Some(pack(ToolApprovalState::Approved, ExecutorReadiness::Ready));
        missing.requested_capability = "tool.write".to_owned();
        assert!(admit_tool_dispatch(&missing).expect_err("capability").contains(&ExecutorIssue::CapabilityGap));

        let denied = request(Some(pack(ToolApprovalState::Denied, ExecutorReadiness::Ready)));
        assert!(admit_tool_dispatch(&denied).expect_err("approval").contains(&ExecutorIssue::DeniedApproval));

        let mut mutating = request(Some(pack(ToolApprovalState::Approved, ExecutorReadiness::Ready)));
        mutating.mutating = true;
        assert!(admit_tool_dispatch(&mutating).expect_err("mutation").contains(&ExecutorIssue::MutationConflict));
    }

    #[test]
    fn fake_executor_rejects_malformed_unbounded_timeout_cancel_and_leaks() {
        // r[verify agent_harness_runtime.tool_executor_packs.receipts.unsafe_reference]
        // r[verify agent_harness_runtime.tool_executor_packs.docs]
        let timed_out = request(Some(pack(ToolApprovalState::Approved, ExecutorReadiness::TimedOut)));
        assert!(admit_tool_dispatch(&timed_out).expect_err("timeout").contains(&ExecutorIssue::TimedOut));

        let cancelled = request(Some(pack(ToolApprovalState::Approved, ExecutorReadiness::Cancelled)));
        assert!(admit_tool_dispatch(&cancelled).expect_err("cancelled").contains(&ExecutorIssue::Cancelled));

        let request = request(Some(pack(ToolApprovalState::Approved, ExecutorReadiness::Ready)));
        let bad = FakeExecutorOutput {
            output_summary: "RAW_TOOL_OUTPUT:body".to_owned(),
            output_bytes: MAX_OUTPUT_BYTES + MIN_COUNT,
            malformed: true,
            unsafe_reference: Some("/home/user/private".to_owned()),
        };
        let issues = run_fake_executor(&request, &bad).expect_err("unsafe output");
        assert!(issues.contains(&ExecutorIssue::MalformedOutput));
        assert!(issues.contains(&ExecutorIssue::UnboundedOutput));
        assert!(issues.contains(&ExecutorIssue::UnsafeReference));
        assert!(issues.contains(&ExecutorIssue::ReceiptLeak));

        let mut receipt = ExecutorReceipt {
            schema: RECEIPT_SCHEMA.to_owned(),
            tool_name: TOOL_NAME.to_owned(),
            executor_label: EXECUTOR_LABEL.to_owned(),
            status: ExecutorOutcome::Completed,
            argument_hash: tool_executor_hash("args", ARGUMENT_BYTES),
            output_hash: tool_executor_hash("output", OUTPUT_SUMMARY.as_bytes()),
            redacted: false,
            issue_codes: Vec::new(),
            receipt_hash: String::new(),
        };
        receipt.receipt_hash = receipt_hash(&receipt);
        assert!(validate_executor_receipt(&receipt).expect_err("receipt leak").contains(&ExecutorIssue::ReceiptLeak));
    }

    fn definition() -> ToolDefinitionMetadata {
        ToolDefinitionMetadata {
            tool_name: TOOL_NAME.to_owned(),
            schema_hash: tool_executor_hash("schema", SCHEMA_BYTES),
            trusted: true,
            active: true,
        }
    }

    fn pack(approval_state: ToolApprovalState, readiness: ExecutorReadiness) -> ToolExecutorPack {
        ToolExecutorPack {
            pack_label: PACK_LABEL.to_owned(),
            tool_name: TOOL_NAME.to_owned(),
            executor_label: EXECUTOR_LABEL.to_owned(),
            required_capability: CAPABILITY.to_owned(),
            output_policy: ToolOutputPolicy::default(),
            mutation_policy: ToolMutationPolicy::ReadOnly,
            approval_state,
            prompt_exposure: PromptExposureState::Exposed,
            readiness,
        }
    }

    fn request(pack: Option<ToolExecutorPack>) -> ToolDispatchRequest {
        ToolDispatchRequest {
            definition: definition(),
            pack,
            requested_capability: CAPABILITY.to_owned(),
            argument_hash: tool_executor_hash("args", ARGUMENT_BYTES),
            mutating: false,
        }
    }
}
