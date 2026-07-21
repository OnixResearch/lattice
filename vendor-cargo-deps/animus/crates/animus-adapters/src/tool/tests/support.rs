use super::super::planning::*;
use super::super::*;

pub const NAME: &str = "search";
pub const INACTIVE_NAME: &str = "write_file";
pub const LABEL: &str = "Search";
pub const OUTPUT: &str = "result";
pub const DESCRIPTION: &str = "Search indexed project context";
pub const SCHEMA_ID: &str = "schema.search.v1";
pub const LEGACY_ARGUMENT: &str = "q";
pub const CURRENT_ARGUMENT: &str = "query";
pub const EXTRA_ARGUMENT: &str = "unexpected";
pub const ARGUMENT_VALUE: &str = "adapter contracts";
pub const PROMPT_SNIPPET: &str = "Use search for indexed project facts.";
pub const PROMPT_GUIDELINE: &str = "Do not infer missing search results.";
pub const SOURCE_IDENTITY: &str = "host.registry";
pub const PROVENANCE: &str = "builtin";
pub const CAPABILITY_READ: &str = "tool.read";
pub const RESULT_SHAPE: &str = "text-summary";
pub const BLANK_TEXT: &str = "   ";
pub const SAFE_REFERENCE: &str = "artifact/search-output";
pub const UNSAFE_REFERENCE: &str = "/tmp/search-output";
pub const LONG_OUTPUT: &str = "first line\nsecond line\nthird line";
pub const VISIBLE_BUDGET: u32 = 10;
pub const LONG_OUTPUT_CHAR_COUNT: u64 = 33;
pub const LONG_OUTPUT_LINE_COUNT: u64 = 3;
pub const CALL_FIRST: &str = "call-1";
pub const CALL_SECOND: &str = "call-2";
pub const CALL_THIRD: &str = "call-3";
pub const CORRELATION_ID: &str = "tool-call-1";
pub const TARGET_FILE: &str = "resource:file:README";
pub const TARGET_OTHER: &str = "resource:file:docs";
pub const PROMPT_CONTEXT_ENTRY_COUNT: usize = 2;
pub const FIRST_INDEX: usize = 0;
pub const SECOND_INDEX: usize = 1;
pub const THIRD_INDEX: usize = 2;
pub const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
pub const HASH_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
pub const HASH_E: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
pub const TAG_FIRST: &str = "L1:aaaa";
pub const TAG_SECOND: &str = "L2:bbbb";
pub const TAG_THIRD: &str = "L3:cccc";
pub const TAG_UNKNOWN: &str = "L9:ffff";
pub const TAG_GENERATION: &str = "blake3-line-ordinal-v1";
pub const RESOURCE_CLASS_FILE: &str = "file";
pub const RESOURCE_ID_HASH: &str = HASH_E;
pub const PRIVATE_RESOURCE_ID: &str = "/home/user/secret.rs";
pub const REPLACEMENT_TEXT: &str = "replacement line";
pub const DELETE_TEXT: &str = "";
pub const NORMALIZATION_LABEL: &str = "lf";
pub const NEWLINE_POLICY_LABEL: &str = "preserve_snapshot";
pub const RESOURCE_REDACTION_HASH_LABEL: &str = "resource_hash";
pub const RESOURCE_REDACTION_RELATIVE_LABEL: &str = "relative_path";
pub const DIAGNOSTIC_APPLIED: &str = "applied";
pub const FIRST_LINE_NUMBER: u32 = 1;
pub const SECOND_LINE_NUMBER: u32 = 2;
pub const THIRD_LINE_NUMBER: u32 = 3;
pub const SNAPSHOT_LINE_COUNT: u32 = THIRD_LINE_NUMBER;
pub const HASHED_LINE_COUNT: usize = 3;
pub const HASHED_EDIT_COUNT: usize = THIRD_INDEX;
pub const SINGLE_HASHED_EDIT_COUNT: usize = crate::MIN_ADAPTER_FIELD_CHARS;
pub const OVERSIZED_REPLACEMENT_CHARS: usize =
    crate::MAX_ADAPTER_FIELD_CHARS.saturating_add(crate::MIN_ADAPTER_FIELD_CHARS);
pub const DISCOVERY_QUERY: &str = "file inspection";
pub const PLAN_BATCH: &str = "batch-search-read";
pub const STEP_FIRST: &str = "step-search";
pub const STEP_SECOND: &str = "step-read";
pub const BINDING_FIRST: &str = "binding-search-summary";
pub const PLAN_BATCH_POLICY_SERIAL: &str = "serial";
pub const TARGET_ARGUMENT: &str = CURRENT_ARGUMENT;
pub const FABRIC_PROFILE: &str = "code-mode-plan-profile";
pub const FABRIC_BUDGET: &str = "small-batch";
pub const FABRIC_KIND_STEEL: &str = "steel";
pub const FABRIC_RECEIPT_POSTURE: &str = "safe_receipt";
pub const FABRIC_ACTION_DISCOVERY: &str = "discovery";
pub const FABRIC_ACTION_EMIT_PLAN: &str = "emit_tool_plan";
pub const FABRIC_ACTION_PROCESS: &str = "process";
pub const DIAGNOSTIC_OK: &str = "tool_plan_admitted";
pub const DIAGNOSTIC_RAW_PROMPT: &str = "prompt: reveal raw planner text";
pub const DETAIL_COUNT: usize = 1;
pub const SUMMARY_COUNT: usize = 1;
pub const SINGLE_STEP_COUNT: usize = 1;
pub const DOUBLE_STEP_COUNT: usize = 2;

pub fn definition(status: DefinitionStatus, name: &str) -> Definition {
    Definition::validate(&definition_draft(status, name)).expect("definition draft should validate")
}

pub fn definition_draft(status: DefinitionStatus, name: &str) -> DefinitionDraft<'_> {
    DefinitionDraft {
        name: Some(name),
        label: Some(LABEL),
        description: Some(DESCRIPTION),
        parameter_schema_identity: Some(SCHEMA_ID),
        prompt_snippet: Some(PROMPT_SNIPPET),
        prompt_guidelines: &[PROMPT_GUIDELINE],
        source_identity: Some(SOURCE_IDENTITY),
        provenance: Some(PROVENANCE),
        capability_labels: &[CAPABILITY_READ],
        result_metadata_shape: Some(RESULT_SHAPE),
        status,
    }
}

pub fn prepared_arguments() -> PreparedArguments {
    let fields = [ArgumentFieldDraft {
        name: Some(CURRENT_ARGUMENT),
        value: Some(ARGUMENT_VALUE),
    }];
    PreparedArguments::prepare(&ArgumentPreparationRequest {
        current_schema_identity: SCHEMA_ID,
        allowed_field_names: &[CURRENT_ARGUMENT],
        compatibility_rules: &[],
        fields: &fields,
    })
    .expect("current arguments should validate")
}

pub fn request() -> Request {
    request_for(RequestFixture {
        name: NAME,
        approval_posture: APPROVAL_PRE_APPROVED,
        mutation_targets: &[TARGET_FILE],
        required_capability_labels: &[CAPABILITY_READ],
    })
}

#[derive(Clone, Copy)]
pub struct RequestFixture<'a> {
    pub name: &'a str,
    pub approval_posture: &'a str,
    pub mutation_targets: &'a [&'a str],
    pub required_capability_labels: &'a [&'a str],
}

pub fn request_for(input: RequestFixture<'_>) -> Request {
    let arguments = prepared_arguments();
    Request::from_draft(&RequestDraft {
        name: Some(input.name),
        arguments: Some(&arguments),
        correlation_id: Some(CORRELATION_ID),
        output_policy: OutputPolicy {
            visible_char_budget: VISIBLE_BUDGET,
        },
        mutation_targets: input.mutation_targets,
        required_capability_labels: input.required_capability_labels,
        approval_posture: input.approval_posture,
    })
    .expect("current-shape request should validate")
}

pub fn discovery() -> DiscoverySet {
    let definitions = [definition(DefinitionStatus::Active, NAME)];
    discover_tools(
        &DiscoveryRequestDraft {
            query_label: Some(DISCOVERY_QUERY),
            required_capability_labels: &[CAPABILITY_READ],
            detail_tool_names: &[NAME],
            ceilings: DiscoveryCeilings::bounded(),
        },
        &definitions,
    )
    .expect("discovery should validate")
}

pub fn admitted_fabric() -> PlanFabricDraft<'static> {
    PlanFabricDraft {
        kind: Some(FABRIC_KIND_STEEL),
        profile_identity: Some(FABRIC_PROFILE),
        policy_hash: Some(HASH_A),
        script_hash: Some(HASH_B),
        allowed_host_actions: &[FABRIC_ACTION_DISCOVERY, FABRIC_ACTION_EMIT_PLAN],
        budget_label: Some(FABRIC_BUDGET),
        receipt_posture: Some(FABRIC_RECEIPT_POSTURE),
    }
}

pub fn response() -> Response {
    let bounded = BoundedOutput::new(
        &OutputDraft {
            body: OUTPUT,
            full_output_reference: None,
            hash_requirement: HashRequirement::Required,
        },
        OutputPolicy {
            visible_char_budget: VISIBLE_BUDGET,
        },
    )
    .expect("bounded output should validate");
    Response::from_bounded(bounded, OutputRedactionClass::Safe)
}

#[derive(Clone, Copy)]
pub struct SpanFixture<'a> {
    pub start_tag: &'a str,
    pub end_tag: &'a str,
    pub replacement: &'a str,
}

pub fn hashed_snapshot() -> hashed::Snapshot {
    hashed::Snapshot::new(&hashed::SnapshotDraft {
        resource_class: RESOURCE_CLASS_FILE,
        resource_identity: RESOURCE_ID_HASH,
        snapshot_hash: HASH_A,
        normalization: NORMALIZATION_LABEL,
        line_count: SNAPSHOT_LINE_COUNT,
        tag_generation: TAG_GENERATION,
        tagged_lines: &hashed_lines(),
    })
    .expect("hashed edit snapshot should validate")
}

pub fn hashed_lines() -> [hashed::TaggedLineDraft<'static>; HASHED_LINE_COUNT] {
    [
        hashed::TaggedLineDraft {
            line_number: FIRST_LINE_NUMBER,
            tag: TAG_FIRST,
            line_hash: HASH_B,
        },
        hashed::TaggedLineDraft {
            line_number: SECOND_LINE_NUMBER,
            tag: TAG_SECOND,
            line_hash: HASH_C,
        },
        hashed::TaggedLineDraft {
            line_number: THIRD_LINE_NUMBER,
            tag: TAG_THIRD,
            line_hash: HASH_D,
        },
    ]
}

pub fn span_draft(input: SpanFixture<'_>) -> hashed::SpanDraft<'_> {
    hashed::SpanDraft {
        start_tag: input.start_tag,
        end_tag: input.end_tag,
        replacement: input.replacement,
    }
}

pub fn hashed_request<'a>(spans: &'a [hashed::SpanDraft<'a>]) -> hashed::RequestDraft<'a> {
    hashed::RequestDraft {
        expected_snapshot_hash: HASH_A,
        current_snapshot_hash: HASH_A,
        newline_policy: NEWLINE_POLICY_LABEL,
        spans,
    }
}
