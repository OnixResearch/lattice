use super::{BLAKE3_ALGORITHM, HASH_PART_SEPARATOR, PROVIDER_HARDENING_HASH_DOMAIN};

const PROVIDER_STREAM_HASH_LABEL: &str = "provider_stream";
const PROVIDER_STREAM_PARSER_CLASS: &str = "provider-sse-json-model-stream";
const PROVIDER_STREAM_ENTRYPOINT: &str = "parse_provider_stream_chunks";
const PROVIDER_STREAM_OUTPUT_TYPE: &str = "animus_adapters::ModelStreamFrame";
const PROVIDER_STREAM_BUDGET_POLICY: &str =
    "frame, record, aggregate, record-count, event-count, completion-count pre-buffer budgets";
const PROVIDER_STREAM_RECEIPT_PROJECTION: &str = "ProviderStreamParserReceipt";
const RAW_BOUNDARY_REVIEW: &str =
    "byte chunks are budgeted before UTF-8 decode and line aggregation";
const INPUT_FRAMING: &str = "UTF-8 chunks containing JSON lines or SSE data: JSON lines";
const JSON_OBJECT_START: char = '{';
const JSON_OBJECT_END: char = '}';
const JSON_QUOTE: char = '"';
const JSON_COLON: char = ':';
const JSON_COMMA: char = ',';
const LINE_FEED: u8 = b'\n';
const LINE_FEED_BYTE_LEN: usize = 1;
const CARRIAGE_RETURN: char = '\r';
const SSE_DATA_PREFIX: &str = "data:";
const JSON_KEY_EVENT: &str = "event";
const JSON_KEY_STATUS: &str = "status";
const JSON_KEY_CALL_ID: &str = "call_id";
const JSON_KEY_ORDER: &str = "order";
const JSON_KEY_BODY: &str = "body";
const JSON_KEY_REASON: &str = "reason";
const JSON_KEY_TOOL_CALL_ID: &str = "tool_call_id";
const JSON_KEY_INPUT_TOKENS: &str = "input_tokens";
const JSON_KEY_OUTPUT_TOKENS: &str = "output_tokens";
const EVENT_CONTENT: &str = "content";
const EVENT_THOUGHT: &str = "thought";
const EVENT_TOOL: &str = "tool";
const EVENT_USAGE: &str = "usage";
const EVENT_FINISH: &str = "finish";
const EVENT_CANCELLED: &str = "cancelled";
const EVENT_FAILURE: &str = "failure";
const EVENT_TERMINAL: &str = "terminal";
const REDACTION_THOUGHT: &str = "thought:redacted";
const REDACTION_TOOL_CALL: &str = "tool_call:redacted";
const REDACTION_PROVIDER_PAYLOAD: &str = "provider_payload:redacted";
const REDACTED_PROVIDER_FAMILY: &str = "redacted_provider_family";
const UNKNOWN_PROVIDER_FAMILY: &str = "unknown_provider_family";
const RECEIPT_HASH_PREFIX: &str = "BLAKE3:";
const DETAIL_EMPTY_STREAM: &str = "provider stream chunks are empty";
const DETAIL_INVALID_UTF8: &str = "provider stream chunk is not valid UTF-8";
const DETAIL_TRUNCATED_RECORD: &str = "provider stream ended with a truncated record";
const DETAIL_MALFORMED_JSON: &str = "provider stream record is malformed JSON";
const DETAIL_MISSING_FIELD: &str = "provider stream record is missing a required field";
const DETAIL_UNSUPPORTED_EVENT: &str = "provider stream event is unsupported";
const DETAIL_UNKNOWN_TERMINAL: &str = "provider stream terminal status is unknown";
const DETAIL_DUPLICATE_TERMINAL: &str = "provider stream contains multiple terminal records";
const DETAIL_LATE_DELTA: &str = "provider stream contains a delta after a terminal record";
const DETAIL_INVALID_ORDER: &str = "provider stream order is not strictly increasing";
const DETAIL_MISSING_TERMINAL: &str = "provider stream is missing a terminal record";
const DETAIL_RECEIPT_LEAK: &str = "provider stream parser receipt contains unsafe material";
const DETAIL_BUDGET_OVERFLOW: &str = "provider stream budget was exceeded before aggregation";
const DEFAULT_FRAME_BYTE_BUDGET: usize = 8_192;
const DEFAULT_RECORD_BYTE_BUDGET: usize = 8_192;
const DEFAULT_AGGREGATE_BYTE_BUDGET: usize = 65_536;
const DEFAULT_RECORD_COUNT_BUDGET: usize = 64;
const DEFAULT_EVENT_COUNT_BUDGET: usize = 64;
const DEFAULT_COMPLETION_COUNT_BUDGET: usize = 1;
const EXPECTED_INVENTORY_ENTRY_COUNT: usize = 1;
const ONE_RECORD: usize = 1;
const ONE_EVENT: usize = 1;
const ONE_COMPLETION: usize = 1;

/// Provider stream parser budget kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderStreamBudgetKind {
    /// A raw inbound chunk exceeded the admitted frame/chunk byte budget.
    FrameBytes,
    /// A pending provider record exceeded the admitted record byte budget.
    RecordBytes,
    /// Total admitted stream bytes exceeded the admitted aggregate byte budget.
    AggregateBytes,
    /// Provider record count exceeded the admitted record count budget.
    RecordCount,
    /// Normalized stream event count exceeded the admitted event count budget.
    EventCount,
    /// Terminal completion count exceeded the admitted completion budget.
    CompletionCount,
}

impl ProviderStreamBudgetKind {
    /// Returns a safe stable label for the budget kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FrameBytes => "frame_bytes",
            Self::RecordBytes => "record_bytes",
            Self::AggregateBytes => "aggregate_bytes",
            Self::RecordCount => "record_count",
            Self::EventCount => "event_count",
            Self::CompletionCount => "completion_count",
        }
    }
}

/// Admitted budgets for provider-stream parsing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderStreamParserBudget {
    /// Maximum bytes in one inbound raw chunk.
    pub frame_bytes: usize,
    /// Maximum bytes in one buffered record before parsing.
    pub record_bytes: usize,
    /// Maximum bytes admitted across all chunks.
    pub aggregate_bytes: usize,
    /// Maximum number of raw records admitted.
    pub record_count: usize,
    /// Maximum number of normalized stream events emitted.
    pub event_count: usize,
    /// Maximum number of terminal completion records admitted.
    pub completion_count: usize,
}

impl Default for ProviderStreamParserBudget {
    fn default() -> Self {
        Self {
            frame_bytes: DEFAULT_FRAME_BYTE_BUDGET,
            record_bytes: DEFAULT_RECORD_BYTE_BUDGET,
            aggregate_bytes: DEFAULT_AGGREGATE_BYTE_BUDGET,
            record_count: DEFAULT_RECORD_COUNT_BUDGET,
            event_count: DEFAULT_EVENT_COUNT_BUDGET,
            completion_count: DEFAULT_COMPLETION_COUNT_BUDGET,
        }
    }
}

/// Borrowed provider-stream parser input.
#[derive(Clone, Copy)]
pub struct ProviderStreamParserInput<'a> {
    /// Safe provider-family label.
    pub provider_family: &'a str,
    /// Raw provider-like byte chunks.
    pub chunks: &'a [&'a [u8]],
    /// Admitted parser budgets.
    pub budget: ProviderStreamParserBudget,
}

/// Stream parser status class recorded in safe receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderStreamParserStatus {
    /// Stream parsed and ended with a successful finish frame.
    Success,
    /// Stream parsed and ended with cancellation.
    Cancelled,
    /// Stream parsed and ended with a provider failure frame.
    ProviderFailure,
    /// Parser rejected the input before a successful terminal event.
    ParserFailure,
}

impl ProviderStreamParserStatus {
    /// Returns a safe stable label for the status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Cancelled => "cancelled",
            Self::ProviderFailure => "provider_failure",
            Self::ParserFailure => "parser_failure",
        }
    }
}

/// Stream parser failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderStreamParseFailureKind {
    /// Raw bytes were not valid UTF-8.
    InvalidEncoding,
    /// A record was malformed or missing required fields.
    MalformedRecord,
    /// An event channel was unsupported.
    UnsupportedEvent,
    /// Record order or terminal position was invalid.
    InvalidOrder,
    /// More than one terminal record was observed.
    DuplicateTerminal,
    /// No terminal record was observed.
    MissingTerminal,
    /// Terminal status was not recognized.
    UnknownTerminalState,
    /// One admitted budget was exceeded before aggregation.
    BudgetExceeded,
    /// Receipt projection or validation detected unsafe material.
    ReceiptLeak,
}

impl ProviderStreamParseFailureKind {
    /// Returns a safe stable label for the failure kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidEncoding => "invalid_encoding",
            Self::MalformedRecord => "malformed_record",
            Self::UnsupportedEvent => "unsupported_event",
            Self::InvalidOrder => "invalid_order",
            Self::DuplicateTerminal => "duplicate_terminal",
            Self::MissingTerminal => "missing_terminal",
            Self::UnknownTerminalState => "unknown_terminal_state",
            Self::BudgetExceeded => "budget_exceeded",
            Self::ReceiptLeak => "receipt_leak",
        }
    }
}

/// Typed parser failure without raw provider data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderStreamParseFailure {
    /// Failure class.
    pub kind: ProviderStreamParseFailureKind,
    /// Optional budget kind when the failure is budget-related.
    pub budget_kind: Option<ProviderStreamBudgetKind>,
    /// Safe detail label.
    pub detail: String,
}

/// Safe parser receipt for success, cancellation, provider failure, or parser failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderStreamParserReceipt {
    /// Parser class.
    pub parser_class: String,
    /// Safe provider-family label.
    pub provider_family: String,
    /// Receipt status.
    pub status: ProviderStreamParserStatus,
    /// Optional failure class.
    pub failure_kind: Option<ProviderStreamParseFailureKind>,
    /// Optional budget kind.
    pub budget_kind: Option<ProviderStreamBudgetKind>,
    /// Raw record count admitted.
    pub record_count: usize,
    /// Normalized event/frame count emitted.
    pub event_count: usize,
    /// Terminal completion record count.
    pub completion_count: usize,
    /// Aggregate bytes admitted.
    pub aggregate_bytes: usize,
    /// BLAKE3 identity over admitted raw bytes.
    pub stream_identity: String,
    /// Redaction markers for sensitive stream channels.
    pub redaction_markers: Vec<String>,
}

/// Successful parser report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderStreamParseReport {
    /// Canonical stream frames.
    pub frames: Vec<animus_adapters::ModelStreamFrame>,
    /// Safe receipt projection.
    pub receipt: ProviderStreamParserReceipt,
}

/// Failed parser report with a safe receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderStreamParseFailureReport {
    /// Typed parser failure.
    pub failure: ProviderStreamParseFailure,
    /// Safe failure receipt.
    pub receipt: ProviderStreamParserReceipt,
}

/// Inventory entry for reviewed stream parser entrypoints.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderStreamParserInventoryEntry {
    /// Entrypoint function name.
    pub entrypoint: String,
    /// Raw input framing accepted by the entrypoint.
    pub input_framing: String,
    /// Normalized output event type.
    pub normalized_output: String,
    /// Budget policy summary.
    pub budget_policy: String,
    /// Receipt projection summary.
    pub receipt_projection: String,
    /// Reviewed raw-boundary assertion and panic posture.
    pub raw_boundary_review: String,
}

/// Returns the reviewed provider-stream parser inventory.
#[must_use]
pub fn provider_stream_parser_inventory() -> Vec<ProviderStreamParserInventoryEntry> {
    vec![ProviderStreamParserInventoryEntry {
        entrypoint: PROVIDER_STREAM_ENTRYPOINT.to_string(),
        input_framing: INPUT_FRAMING.to_string(),
        normalized_output: PROVIDER_STREAM_OUTPUT_TYPE.to_string(),
        budget_policy: PROVIDER_STREAM_BUDGET_POLICY.to_string(),
        receipt_projection: PROVIDER_STREAM_RECEIPT_PROJECTION.to_string(),
        raw_boundary_review: RAW_BOUNDARY_REVIEW.to_string(),
    }]
}

/// Validates parser inventory completeness.
///
/// # Errors
///
/// Returns diagnostics when reviewed parser inventory entries are absent or incomplete.
pub fn validate_provider_stream_parser_inventory(
    entries: &[ProviderStreamParserInventoryEntry],
) -> Result<(), Vec<super::data::ProviderDiagnostic>> {
    let mut diagnostics = Vec::new();
    if entries.len() != EXPECTED_INVENTORY_ENTRY_COUNT {
        diagnostics.push(inventory_diagnostic("entry_count"));
    }
    for entry in entries {
        collect_blank_inventory_field(entry.entrypoint.as_str(), "entrypoint", &mut diagnostics);
        collect_blank_inventory_field(
            entry.input_framing.as_str(),
            "input_framing",
            &mut diagnostics,
        );
        collect_blank_inventory_field(
            entry.normalized_output.as_str(),
            "normalized_output",
            &mut diagnostics,
        );
        collect_blank_inventory_field(
            entry.budget_policy.as_str(),
            "budget_policy",
            &mut diagnostics,
        );
        collect_blank_inventory_field(
            entry.receipt_projection.as_str(),
            "receipt_projection",
            &mut diagnostics,
        );
        collect_blank_inventory_field(
            entry.raw_boundary_review.as_str(),
            "raw_boundary_review",
            &mut diagnostics,
        );
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// Parses raw provider-like stream chunks into canonical model stream frames.
///
/// # Errors
///
/// Returns a typed parser failure report with a safe receipt when input is malformed,
/// out of order, over budget, unsupported, or unsafe to project.
pub fn parse_provider_stream_chunks(
    input: ProviderStreamParserInput<'_>,
) -> Result<ProviderStreamParseReport, Box<ProviderStreamParseFailureReport>> {
    let mut state = ParserState::new(input.provider_family, input.budget);
    if input.chunks.is_empty() {
        return Err(Box::new(state.failure_report(failure(
            ProviderStreamParseFailureKind::MalformedRecord,
            None,
            DETAIL_EMPTY_STREAM,
        ))));
    }
    for chunk in input.chunks {
        if let Err(error) = state.push_chunk(chunk) {
            return Err(Box::new(state.failure_report(error)));
        }
    }
    match state.finish() {
        Ok(report) => Ok(report),
        Err(error) => Err(Box::new(state.failure_report(error))),
    }
}

/// Validates that a stream parser receipt contains only safe summary material.
///
/// # Errors
///
/// Returns a receipt-leak failure when the receipt includes raw payload-shaped material,
/// unknown redaction markers, missing safe labels, or a non-BLAKE3 identity.
pub fn validate_provider_stream_parser_receipt(
    receipt: &ProviderStreamParserReceipt,
) -> Result<(), ProviderStreamParseFailure> {
    if receipt.parser_class.trim().is_empty()
        || !is_safe_provider_family(receipt.provider_family.as_str())
        || !receipt.stream_identity.starts_with(RECEIPT_HASH_PREFIX)
    {
        return Err(failure(
            ProviderStreamParseFailureKind::ReceiptLeak,
            None,
            DETAIL_RECEIPT_LEAK,
        ));
    }
    for marker in &receipt.redaction_markers {
        if !is_allowed_redaction_marker(marker.as_str()) {
            return Err(failure(
                ProviderStreamParseFailureKind::ReceiptLeak,
                None,
                DETAIL_RECEIPT_LEAK,
            ));
        }
    }
    Ok(())
}

struct ParserState<'a> {
    provider_family: &'a str,
    budget: ProviderStreamParserBudget,
    buffer: String,
    frames: Vec<animus_adapters::ModelStreamFrame>,
    record_count: usize,
    event_count: usize,
    completion_count: usize,
    aggregate_bytes: usize,
    terminal_seen: bool,
    previous_order: Option<u64>,
    redactions: RedactionFlags,
    hasher: blake3::Hasher,
}

struct RedactionFlags {
    thought: bool,
    tool: bool,
    provider_payload: bool,
}

impl<'a> ParserState<'a> {
    fn new(provider_family: &'a str, budget: ProviderStreamParserBudget) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(PROVIDER_HARDENING_HASH_DOMAIN.as_bytes());
        hasher.update(HASH_PART_SEPARATOR.as_bytes());
        hasher.update(PROVIDER_STREAM_HASH_LABEL.as_bytes());
        hasher.update(HASH_PART_SEPARATOR.as_bytes());
        Self {
            provider_family,
            budget,
            buffer: String::new(),
            frames: Vec::new(),
            record_count: usize::default(),
            event_count: usize::default(),
            completion_count: usize::default(),
            aggregate_bytes: usize::default(),
            terminal_seen: false,
            previous_order: None,
            redactions: RedactionFlags {
                thought: false,
                tool: false,
                provider_payload: false,
            },
            hasher,
        }
    }

    fn push_chunk(&mut self, chunk: &[u8]) -> Result<(), ProviderStreamParseFailure> {
        Self::check_limit(
            chunk.len(),
            self.budget.frame_bytes,
            ProviderStreamBudgetKind::FrameBytes,
        )?;
        let next_aggregate = self
            .aggregate_bytes
            .checked_add(chunk.len())
            .ok_or_else(|| budget_failure(ProviderStreamBudgetKind::AggregateBytes))?;
        Self::check_limit(
            next_aggregate,
            self.budget.aggregate_bytes,
            ProviderStreamBudgetKind::AggregateBytes,
        )?;
        self.aggregate_bytes = next_aggregate;
        self.hasher.update(chunk);
        let text = core::str::from_utf8(chunk).map_err(|_error| {
            failure(
                ProviderStreamParseFailureKind::InvalidEncoding,
                None,
                DETAIL_INVALID_UTF8,
            )
        })?;
        self.push_text(text)
    }

    fn push_text(&mut self, text: &str) -> Result<(), ProviderStreamParseFailure> {
        let mut start = usize::default();
        for (index, byte) in text.bytes().enumerate() {
            if byte == LINE_FEED {
                let end = index.saturating_add(LINE_FEED_BYTE_LEN);
                self.push_segment(&text[start..end])?;
                start = end;
            }
        }
        if start < text.len() {
            self.push_segment(&text[start..])?;
        }
        Ok(())
    }

    fn push_segment(&mut self, segment: &str) -> Result<(), ProviderStreamParseFailure> {
        let next_record_bytes = self
            .buffer
            .len()
            .checked_add(segment.len())
            .ok_or_else(|| budget_failure(ProviderStreamBudgetKind::RecordBytes))?;
        Self::check_limit(
            next_record_bytes,
            self.budget.record_bytes,
            ProviderStreamBudgetKind::RecordBytes,
        )?;
        self.buffer.push_str(segment);
        if segment.as_bytes().last().copied() == Some(LINE_FEED) {
            self.finish_record()?;
        }
        Ok(())
    }

    fn finish_record(&mut self) -> Result<(), ProviderStreamParseFailure> {
        let owned_record = self
            .buffer
            .trim_end_matches('\n')
            .trim_end_matches(CARRIAGE_RETURN)
            .trim()
            .to_string();
        self.buffer.clear();
        if owned_record.is_empty() {
            return Ok(());
        }
        let next_record_count = self
            .record_count
            .checked_add(ONE_RECORD)
            .ok_or_else(|| budget_failure(ProviderStreamBudgetKind::RecordCount))?;
        Self::check_limit(
            next_record_count,
            self.budget.record_count,
            ProviderStreamBudgetKind::RecordCount,
        )?;
        self.record_count = next_record_count;
        let frame = record_frame(owned_record.as_str())?;
        self.push_frame(frame)
    }

    fn push_frame(
        &mut self,
        frame: animus_adapters::ModelStreamFrame,
    ) -> Result<(), ProviderStreamParseFailure> {
        let kind = frame.kind();
        if self.terminal_seen {
            return if kind.is_terminal() {
                Err(failure(
                    ProviderStreamParseFailureKind::DuplicateTerminal,
                    None,
                    DETAIL_DUPLICATE_TERMINAL,
                ))
            } else {
                Err(failure(
                    ProviderStreamParseFailureKind::InvalidOrder,
                    None,
                    DETAIL_LATE_DELTA,
                ))
            };
        }
        self.check_order(frame.order().value())?;
        let next_event_count = self
            .event_count
            .checked_add(ONE_EVENT)
            .ok_or_else(|| budget_failure(ProviderStreamBudgetKind::EventCount))?;
        Self::check_limit(
            next_event_count,
            self.budget.event_count,
            ProviderStreamBudgetKind::EventCount,
        )?;
        if kind.is_terminal() {
            let next_completion_count = self
                .completion_count
                .checked_add(ONE_COMPLETION)
                .ok_or_else(|| budget_failure(ProviderStreamBudgetKind::CompletionCount))?;
            Self::check_limit(
                next_completion_count,
                self.budget.completion_count,
                ProviderStreamBudgetKind::CompletionCount,
            )?;
            self.completion_count = next_completion_count;
            self.terminal_seen = true;
        }
        self.collect_redaction(kind);
        self.previous_order = Some(frame.order().value());
        self.event_count = next_event_count;
        self.frames.push(frame);
        Ok(())
    }

    fn check_order(&self, order: u64) -> Result<(), ProviderStreamParseFailure> {
        if self.previous_order.is_none_or(|previous| previous < order) {
            Ok(())
        } else {
            Err(failure(
                ProviderStreamParseFailureKind::InvalidOrder,
                None,
                DETAIL_INVALID_ORDER,
            ))
        }
    }

    fn collect_redaction(&mut self, kind: animus_adapters::ModelStreamFrameKind) {
        match kind {
            animus_adapters::ModelStreamFrameKind::ThoughtDelta => {
                self.redactions.thought = true;
            }
            animus_adapters::ModelStreamFrameKind::ToolCallDelta => {
                self.redactions.tool = true;
                self.redactions.provider_payload = true;
            }
            animus_adapters::ModelStreamFrameKind::ContentDelta
            | animus_adapters::ModelStreamFrameKind::Usage
            | animus_adapters::ModelStreamFrameKind::Finish
            | animus_adapters::ModelStreamFrameKind::Cancelled
            | animus_adapters::ModelStreamFrameKind::Failure => {}
        }
    }

    fn finish(&mut self) -> Result<ProviderStreamParseReport, ProviderStreamParseFailure> {
        if !self.buffer.is_empty() {
            return Err(failure(
                ProviderStreamParseFailureKind::MalformedRecord,
                None,
                DETAIL_TRUNCATED_RECORD,
            ));
        }
        animus_adapters::validate_model_stream_frames(&self.frames)
            .map_err(|error| model_stream_failure(&error))?;
        let status = terminal_status(&self.frames)?;
        let receipt = self.receipt(status, None, None);
        validate_provider_stream_parser_receipt(&receipt)?;
        Ok(ProviderStreamParseReport {
            frames: self.frames.clone(),
            receipt,
        })
    }

    fn failure_report(
        &self,
        failure: ProviderStreamParseFailure,
    ) -> ProviderStreamParseFailureReport {
        ProviderStreamParseFailureReport {
            receipt: self.receipt(
                ProviderStreamParserStatus::ParserFailure,
                Some(failure.kind),
                failure.budget_kind,
            ),
            failure,
        }
    }

    fn receipt(
        &self,
        status: ProviderStreamParserStatus,
        failure_kind: Option<ProviderStreamParseFailureKind>,
        budget_kind: Option<ProviderStreamBudgetKind>,
    ) -> ProviderStreamParserReceipt {
        ProviderStreamParserReceipt {
            parser_class: PROVIDER_STREAM_PARSER_CLASS.to_string(),
            provider_family: safe_provider_family(self.provider_family),
            status,
            failure_kind,
            budget_kind,
            record_count: self.record_count,
            event_count: self.event_count,
            completion_count: self.completion_count,
            aggregate_bytes: self.aggregate_bytes,
            stream_identity: self.stream_identity(),
            redaction_markers: self.redaction_markers(),
        }
    }

    fn stream_identity(&self) -> String {
        format!(
            "{BLAKE3_ALGORITHM}:{}",
            self.hasher.clone().finalize().to_hex()
        )
    }

    fn redaction_markers(&self) -> Vec<String> {
        let mut markers = Vec::new();
        if self.redactions.thought {
            markers.push(REDACTION_THOUGHT.to_string());
        }
        if self.redactions.tool {
            markers.push(REDACTION_TOOL_CALL.to_string());
        }
        if self.redactions.provider_payload {
            markers.push(REDACTION_PROVIDER_PAYLOAD.to_string());
        }
        markers
    }

    fn check_limit(
        actual: usize,
        limit: usize,
        kind: ProviderStreamBudgetKind,
    ) -> Result<(), ProviderStreamParseFailure> {
        if actual <= limit {
            Ok(())
        } else {
            Err(budget_failure(kind))
        }
    }
}

fn record_frame(
    record: &str,
) -> Result<animus_adapters::ModelStreamFrame, ProviderStreamParseFailure> {
    let json = normalized_json_record(record)?;
    let event = json_string_field(json, JSON_KEY_EVENT)?;
    let normalized_event = if event == EVENT_TERMINAL {
        json_string_field(json, JSON_KEY_STATUS)?
    } else {
        event
    };
    let call_id = json_string_field(json, JSON_KEY_CALL_ID)?;
    let order = json_u64_field(json, JSON_KEY_ORDER)?;
    match normalized_event {
        EVENT_CONTENT => text_frame(
            call_id,
            order,
            json,
            animus_adapters::ModelStreamFrameKind::ContentDelta,
        ),
        EVENT_THOUGHT => text_frame(
            call_id,
            order,
            json,
            animus_adapters::ModelStreamFrameKind::ThoughtDelta,
        ),
        EVENT_TOOL => tool_frame(call_id, order, json),
        EVENT_USAGE => usage_frame(call_id, order, json),
        EVENT_FINISH => terminal_frame(
            call_id,
            order,
            animus_adapters::ModelStreamFrameKind::Finish,
            None,
        ),
        EVENT_CANCELLED => terminal_frame(
            call_id,
            order,
            animus_adapters::ModelStreamFrameKind::Cancelled,
            Some(json_string_field(json, JSON_KEY_REASON)?),
        ),
        EVENT_FAILURE => terminal_frame(
            call_id,
            order,
            animus_adapters::ModelStreamFrameKind::Failure,
            Some(json_string_field(json, JSON_KEY_REASON)?),
        ),
        _unknown if event == EVENT_TERMINAL => Err(failure(
            ProviderStreamParseFailureKind::UnknownTerminalState,
            None,
            DETAIL_UNKNOWN_TERMINAL,
        )),
        _unknown => Err(failure(
            ProviderStreamParseFailureKind::UnsupportedEvent,
            None,
            DETAIL_UNSUPPORTED_EVENT,
        )),
    }
}

fn normalized_json_record(record: &str) -> Result<&str, ProviderStreamParseFailure> {
    let candidate = record
        .strip_prefix(SSE_DATA_PREFIX)
        .map_or(record, str::trim);
    if candidate.starts_with(JSON_OBJECT_START) && candidate.ends_with(JSON_OBJECT_END) {
        Ok(candidate)
    } else {
        Err(failure(
            ProviderStreamParseFailureKind::MalformedRecord,
            None,
            DETAIL_MALFORMED_JSON,
        ))
    }
}

fn text_frame(
    call_id: &str,
    order: u64,
    json: &str,
    kind: animus_adapters::ModelStreamFrameKind,
) -> Result<animus_adapters::ModelStreamFrame, ProviderStreamParseFailure> {
    let visibility = match kind {
        animus_adapters::ModelStreamFrameKind::ContentDelta => {
            animus_adapters::ModelStreamVisibility::AssistantVisible
        }
        animus_adapters::ModelStreamFrameKind::ThoughtDelta
        | animus_adapters::ModelStreamFrameKind::ToolCallDelta
        | animus_adapters::ModelStreamFrameKind::Usage
        | animus_adapters::ModelStreamFrameKind::Finish
        | animus_adapters::ModelStreamFrameKind::Cancelled
        | animus_adapters::ModelStreamFrameKind::Failure => {
            animus_adapters::ModelStreamVisibility::Redacted
        }
    };
    animus_adapters::ModelStreamFrame::from_draft(animus_adapters::ModelStreamFrameDraft {
        call_id,
        order,
        kind,
        body: Some(json_string_field(json, JSON_KEY_BODY)?),
        tool_call_id: None,
        usage: None,
        visibility,
    })
    .map_err(|error| model_frame_failure(&error))
}

fn tool_frame(
    call_id: &str,
    order: u64,
    json: &str,
) -> Result<animus_adapters::ModelStreamFrame, ProviderStreamParseFailure> {
    animus_adapters::ModelStreamFrame::from_draft(animus_adapters::ModelStreamFrameDraft {
        call_id,
        order,
        kind: animus_adapters::ModelStreamFrameKind::ToolCallDelta,
        body: Some(json_string_field(json, JSON_KEY_BODY)?),
        tool_call_id: Some(json_string_field(json, JSON_KEY_TOOL_CALL_ID)?),
        usage: None,
        visibility: animus_adapters::ModelStreamVisibility::Redacted,
    })
    .map_err(|error| model_frame_failure(&error))
}

fn usage_frame(
    call_id: &str,
    order: u64,
    json: &str,
) -> Result<animus_adapters::ModelStreamFrame, ProviderStreamParseFailure> {
    let input_tokens = json_u32_field(json, JSON_KEY_INPUT_TOKENS)?;
    let output_tokens = json_u32_field(json, JSON_KEY_OUTPUT_TOKENS)?;
    animus_adapters::ModelStreamFrame::from_draft(animus_adapters::ModelStreamFrameDraft {
        call_id,
        order,
        kind: animus_adapters::ModelStreamFrameKind::Usage,
        body: None,
        tool_call_id: None,
        usage: Some(animus_adapters::ModelStreamUsage::new(
            input_tokens,
            output_tokens,
        )),
        visibility: animus_adapters::ModelStreamVisibility::Redacted,
    })
    .map_err(|error| model_frame_failure(&error))
}

fn terminal_frame(
    call_id: &str,
    order: u64,
    kind: animus_adapters::ModelStreamFrameKind,
    body: Option<&str>,
) -> Result<animus_adapters::ModelStreamFrame, ProviderStreamParseFailure> {
    animus_adapters::ModelStreamFrame::from_draft(animus_adapters::ModelStreamFrameDraft {
        call_id,
        order,
        kind,
        body,
        tool_call_id: None,
        usage: None,
        visibility: animus_adapters::ModelStreamVisibility::Redacted,
    })
    .map_err(|error| model_frame_failure(&error))
}

fn json_string_field<'a>(json: &'a str, key: &str) -> Result<&'a str, ProviderStreamParseFailure> {
    let value = json_field_value(json, key)?;
    if !value.starts_with(JSON_QUOTE) {
        return Err(missing_or_malformed_field());
    }
    let rest = &value[JSON_QUOTE.len_utf8()..];
    let Some(end) = rest.find(JSON_QUOTE) else {
        return Err(missing_or_malformed_field());
    };
    Ok(&rest[..end])
}

fn json_u64_field(json: &str, key: &str) -> Result<u64, ProviderStreamParseFailure> {
    json_number_field(json, key).and_then(|raw| {
        raw.parse::<u64>()
            .map_err(|_error| missing_or_malformed_field())
    })
}

fn json_u32_field(json: &str, key: &str) -> Result<u32, ProviderStreamParseFailure> {
    json_number_field(json, key).and_then(|raw| {
        raw.parse::<u32>()
            .map_err(|_error| missing_or_malformed_field())
    })
}

fn json_number_field<'a>(json: &'a str, key: &str) -> Result<&'a str, ProviderStreamParseFailure> {
    let value = json_field_value(json, key)?;
    let end = value
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(value.len());
    if end == usize::default() {
        return Err(missing_or_malformed_field());
    }
    let tail = value[end..].trim_start();
    if tail.starts_with(JSON_COMMA) || tail.starts_with(JSON_OBJECT_END) {
        Ok(&value[..end])
    } else {
        Err(missing_or_malformed_field())
    }
}

fn json_field_value<'a>(json: &'a str, key: &str) -> Result<&'a str, ProviderStreamParseFailure> {
    let field = format!("{JSON_QUOTE}{key}{JSON_QUOTE}");
    let Some(start) = json.find(field.as_str()) else {
        return Err(missing_or_malformed_field());
    };
    let rest = &json[start + field.len()..];
    let Some(colon) = rest.find(JSON_COLON) else {
        return Err(missing_or_malformed_field());
    };
    Ok(rest[colon + JSON_COLON.len_utf8()..].trim_start())
}

fn terminal_status(
    frames: &[animus_adapters::ModelStreamFrame],
) -> Result<ProviderStreamParserStatus, ProviderStreamParseFailure> {
    let Some(frame) = frames.last() else {
        return Err(failure(
            ProviderStreamParseFailureKind::MissingTerminal,
            None,
            DETAIL_MISSING_TERMINAL,
        ));
    };
    match frame.kind() {
        animus_adapters::ModelStreamFrameKind::Finish => Ok(ProviderStreamParserStatus::Success),
        animus_adapters::ModelStreamFrameKind::Cancelled => {
            Ok(ProviderStreamParserStatus::Cancelled)
        }
        animus_adapters::ModelStreamFrameKind::Failure => {
            Ok(ProviderStreamParserStatus::ProviderFailure)
        }
        animus_adapters::ModelStreamFrameKind::ContentDelta
        | animus_adapters::ModelStreamFrameKind::ThoughtDelta
        | animus_adapters::ModelStreamFrameKind::ToolCallDelta
        | animus_adapters::ModelStreamFrameKind::Usage => Err(failure(
            ProviderStreamParseFailureKind::MissingTerminal,
            None,
            DETAIL_MISSING_TERMINAL,
        )),
    }
}

fn model_stream_failure(error: &animus_adapters::ModelStreamError) -> ProviderStreamParseFailure {
    match error {
        animus_adapters::ModelStreamError::EmptyStream
        | animus_adapters::ModelStreamError::MissingTerminalFrame => failure(
            ProviderStreamParseFailureKind::MissingTerminal,
            None,
            DETAIL_MISSING_TERMINAL,
        ),
        animus_adapters::ModelStreamError::OutOfOrderFrame { .. }
        | animus_adapters::ModelStreamError::TerminalFrameNotLast { .. }
        | animus_adapters::ModelStreamError::MixedModelCall { .. } => failure(
            ProviderStreamParseFailureKind::InvalidOrder,
            None,
            DETAIL_INVALID_ORDER,
        ),
        animus_adapters::ModelStreamError::MalformedFixture { .. }
        | animus_adapters::ModelStreamError::InvalidField { .. }
        | animus_adapters::ModelStreamError::InvalidOrder { .. }
        | animus_adapters::ModelStreamError::MissingBody { .. }
        | animus_adapters::ModelStreamError::UnexpectedBody { .. }
        | animus_adapters::ModelStreamError::MissingToolCallId
        | animus_adapters::ModelStreamError::UnexpectedToolCallId { .. }
        | animus_adapters::ModelStreamError::MissingUsage
        | animus_adapters::ModelStreamError::UnexpectedUsage { .. }
        | animus_adapters::ModelStreamError::TerminalFrameNotFinish { .. } => failure(
            ProviderStreamParseFailureKind::MalformedRecord,
            None,
            DETAIL_MALFORMED_JSON,
        ),
        animus_adapters::ModelStreamError::ThoughtVisibilityLeak => failure(
            ProviderStreamParseFailureKind::ReceiptLeak,
            None,
            DETAIL_RECEIPT_LEAK,
        ),
    }
}

fn model_frame_failure(error: &animus_adapters::ModelStreamError) -> ProviderStreamParseFailure {
    model_stream_failure(error)
}

fn budget_failure(kind: ProviderStreamBudgetKind) -> ProviderStreamParseFailure {
    failure(
        ProviderStreamParseFailureKind::BudgetExceeded,
        Some(kind),
        DETAIL_BUDGET_OVERFLOW,
    )
}

fn missing_or_malformed_field() -> ProviderStreamParseFailure {
    failure(
        ProviderStreamParseFailureKind::MalformedRecord,
        None,
        DETAIL_MISSING_FIELD,
    )
}

fn failure(
    kind: ProviderStreamParseFailureKind,
    budget_kind: Option<ProviderStreamBudgetKind>,
    detail: &str,
) -> ProviderStreamParseFailure {
    ProviderStreamParseFailure {
        kind,
        budget_kind,
        detail: detail.to_string(),
    }
}

fn safe_provider_family(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        UNKNOWN_PROVIDER_FAMILY.to_string()
    } else if is_safe_provider_family(trimmed) {
        trimmed.to_string()
    } else {
        REDACTED_PROVIDER_FAMILY.to_string()
    }
}

fn is_safe_provider_family(value: &str) -> bool {
    !value.trim().is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn is_allowed_redaction_marker(marker: &str) -> bool {
    matches!(
        marker,
        REDACTION_THOUGHT | REDACTION_TOOL_CALL | REDACTION_PROVIDER_PAYLOAD
    )
}

fn collect_blank_inventory_field(
    value: &str,
    field: &str,
    diagnostics: &mut Vec<super::data::ProviderDiagnostic>,
) {
    if value.trim().is_empty() {
        diagnostics.push(inventory_diagnostic(field));
    }
}

fn inventory_diagnostic(field: &str) -> super::data::ProviderDiagnostic {
    super::data::ProviderDiagnostic::new(
        super::data::ProviderDiagnosticKind::InvalidCapabilityMetadata,
        field,
        "provider stream parser inventory is incomplete",
    )
}
