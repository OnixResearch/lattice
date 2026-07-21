use super::*;

const USER_ID: &str = "entry-user";
const ASSISTANT_ID: &str = "entry-assistant";
const TOOL_ID: &str = "entry-tool";
const SUMMARY_ID: &str = "entry-summary";
const USAGE_ID: &str = "entry-usage";
const RECEIPT_ID: &str = "entry-receipt";
const UNSUPPORTED_ID: &str = "entry-unsupported";
const ACTOR_KIND: &str = "human";
const ACTOR_LABEL: &str = "operator";
const SESSION_LABEL: &str = "session-a";
const TRACE_LABEL: &str = "trace-a";
const USER_CONTENT: &str = "please inspect bounded provenance";
const ASSISTANT_CONTENT: &str = "inspection recorded";
const TOOL_CONTENT: &str = "bounded tool result";
const SUMMARY_CONTENT: &str = "bounded summary";
const MODEL_LABEL: &str = "model-safe";
const TOOL_KIND: &str = "search";
const ERROR_CLASS: &str = "redaction-error";
const REQUIREMENT_ID: &str = "agent_harness_runtime.provenance_ledger.session_ledger.replay";
const REQUEST_SHAPE: &str = "chat-completion";
const EFFECT_ABILITY: &str = "read";
const EFFECT_RESOURCE: &str = "workspace-doc";
const SERVICE_LABEL: &str = "tool-service";
const RECEIPT_STATUS: &str = "ok";
const SAFE_ISSUE: &str = "ISSUE_SAFE";
const SAFE_MUTATION: &str = "none";
const SAFE_INTERCHANGE: &str = "toolpath-redacted";
const RAW_PROMPT_LEAK: &str = "RAW_PROMPT: copy private request";
const PROVIDER_JSON_LEAK: &str = "provider_json { token }";
const CREDENTIAL_LEAK: &str = "API_KEY=sk-secret";
const FULL_PATH_LEAK: &str = "/home/operator/private";
const TOOL_INPUT_LEAK: &str = "TOOL_INPUT: secret args";
const TOOL_OUTPUT_LEAK: &str = "TOOL_OUTPUT: secret output";
const MODEL_OUTPUT_LEAK: &str = "MODEL_OUTPUT: raw text";
const UNBOUNDED_OUTPUT_LEAK: &str = "UNBOUNDED_OUTPUT: huge";
const RAW_DIFF_LEAK: &str = "unified diff raw body";
const INPUT_TOKENS: u64 = 13;
const OUTPUT_TOKENS: u64 = 21;
const RECEIPT_EVENT_COUNT: u64 = 3;
const OVERSIZED_COUNT: u64 = MAX_RECEIPT_COUNT + 1;
const EXPECTED_REPLAY_MESSAGES: usize = 4;
const EXPECTED_USER_MESSAGES: u32 = 1;
const EXPECTED_ASSISTANT_MESSAGES: u32 = 1;
const EXPECTED_TOOL_MESSAGES: u32 = 1;
const EXPECTED_SUMMARIES: u32 = 1;
const EXPECTED_USAGE_OBSERVATIONS: u32 = 1;
const EXPECTED_RECEIPTS: u32 = 1;
const ORIGINAL_OUTPUT_BYTES: u64 = 512;
const RETAINED_OUTPUT_BYTES: u64 = 128;

fn hash(value: &str) -> Text {
    blake3_hex(value.as_bytes())
}

fn actor() -> ProvenanceActor {
    ProvenanceActor::new(ACTOR_KIND, ACTOR_LABEL).expect("safe actor should validate")
}

fn public_artifact(payload: &str) -> ProvenanceArtifact {
    ProvenanceArtifact::from_draft(&ProvenanceArtifactDraft {
        kind: ProvenanceArtifactKind::Documentation,
        redaction: ProvenanceRedactionClass::Public,
        normalized_payload: payload,
        dependencies: &[],
    })
    .expect("artifact should validate")
}

fn redacted_artifact(payload: &str) -> ProvenanceArtifact {
    ProvenanceArtifact::from_draft(&ProvenanceArtifactDraft {
        kind: ProvenanceArtifactKind::Tool,
        redaction: ProvenanceRedactionClass::RedactedPayload,
        normalized_payload: payload,
        dependencies: &[],
    })
    .expect("artifact should validate")
}

fn receipt(artifact_hash: &str) -> ProvenanceReceipt {
    ProvenanceReceipt::validate(&ProvenanceReceiptDraft {
        id: RECEIPT_ID,
        parents: &[USAGE_ID],
        status: RECEIPT_STATUS,
        issue_code: Some(SAFE_ISSUE),
        actor_label: ACTOR_LABEL,
        session_label: SESSION_LABEL,
        trace_label: TRACE_LABEL,
        event_count: RECEIPT_EVENT_COUNT,
        artifact_hashes: &[artifact_hash],
        mutation_kind: Some(SAFE_MUTATION),
        effect_kind: Some(EFFECT_ABILITY),
        backend_label: Some(SERVICE_LABEL),
        interchange_label: Some(SAFE_INTERCHANGE),
        redaction_markers: &[REDACTED_PAYLOAD_MARKER],
    })
    .expect("receipt should validate")
}

fn ledger_entries() -> Collection<ProvenanceLedgerEntry> {
    let artifact = public_artifact("safe payload");
    vec![
        ProvenanceLedgerEntry::Message(ProvenanceMessageEntry {
            id: USER_ID.to_owned(),
            parents: Collection::new(),
            role: ProvenanceReplayRole::User,
            actor: actor(),
            content: USER_CONTENT.to_owned(),
            content_hash: hash(USER_CONTENT),
        }),
        ProvenanceLedgerEntry::Message(ProvenanceMessageEntry {
            id: ASSISTANT_ID.to_owned(),
            parents: vec![USER_ID.to_owned()],
            role: ProvenanceReplayRole::Assistant,
            actor: ProvenanceActor::new("agent", "assistant").expect("safe actor"),
            content: ASSISTANT_CONTENT.to_owned(),
            content_hash: hash(ASSISTANT_CONTENT),
        }),
        ProvenanceLedgerEntry::Message(ProvenanceMessageEntry {
            id: TOOL_ID.to_owned(),
            parents: vec![ASSISTANT_ID.to_owned()],
            role: ProvenanceReplayRole::Tool,
            actor: ProvenanceActor::new("tool", TOOL_KIND).expect("safe actor"),
            content: TOOL_CONTENT.to_owned(),
            content_hash: hash(TOOL_CONTENT),
        }),
        ProvenanceLedgerEntry::Summary(ProvenanceSummaryEntry {
            id: SUMMARY_ID.to_owned(),
            parents: vec![TOOL_ID.to_owned()],
            summary: SUMMARY_CONTENT.to_owned(),
            summary_hash: hash(SUMMARY_CONTENT),
        }),
        ProvenanceLedgerEntry::Usage(ProvenanceUsageEntry {
            id: USAGE_ID.to_owned(),
            parents: vec![SUMMARY_ID.to_owned()],
            model_label: MODEL_LABEL.to_owned(),
            input_tokens: INPUT_TOKENS,
            output_tokens: OUTPUT_TOKENS,
        }),
        ProvenanceLedgerEntry::Receipt(receipt(artifact.artifact_hash.as_str())),
    ]
}
