//! Optional multi-lens analysis deck contracts.
//!
//! Hosts can define focus decks, assemble bounded task packets, run lens workers,
//! and pass reports back for synthesis. This module validates and reduces typed
//! data only; it never spawns subagents, calls models, reads repositories,
//! opens networks, creates processes, or enforces parallelism.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::assertion_density,
        tigerstyle::bool_naming,
        tigerstyle::function_length,
        tigerstyle::non_trait_imports,
        tigerstyle::numeric_units,
        tigerstyle::path_segment_repetition
    )
)]

use crate::artifact::Hash;
use crate::validation::{self, AdapterContractError, FieldName};

const DECK_ID_FIELD: FieldName = FieldName::new("lens_deck_id");
const DECK_FAMILY_FIELD: FieldName = FieldName::new("lens_deck_family");
const FOCUS_ID_FIELD: FieldName = FieldName::new("lens_focus_id");
const FOCUS_DESCRIPTION_FIELD: FieldName = FieldName::new("lens_focus_description");
const LOOK_FOR_FIELD: FieldName = FieldName::new("lens_look_for");
const RETRIEVAL_BUDGET_FIELD: FieldName = FieldName::new("lens_retrieval_budget_ref");
const EVIDENCE_EXPECTATION_FIELD: FieldName = FieldName::new("lens_evidence_expectation");
const OUTPUT_CONTRACT_FIELD: FieldName = FieldName::new("lens_output_contract");
const CRITERION_ID_FIELD: FieldName = FieldName::new("lens_criterion_id");
const CONTEXT_LABEL_FIELD: FieldName = FieldName::new("lens_context_label");
const REPORT_ID_FIELD: FieldName = FieldName::new("lens_report_id");
const SUMMARY_HASH_FIELD: FieldName = FieldName::new("lens_summary_hash");
const ASSUMPTION_FIELD: FieldName = FieldName::new("lens_assumption");
const CONTRADICTION_ID_FIELD: FieldName = FieldName::new("lens_contradiction_id");
const GAP_ID_FIELD: FieldName = FieldName::new("lens_gap_id");

const MAX_FOCUS_ITEMS: usize = 16;
const MAX_REVIEW_CRITERIA: usize = 12;
const MAX_CONTEXT_REFS: usize = 12;
const MAX_EVIDENCE_HASHES: usize = 16;
const MAX_ASSUMPTIONS: usize = 8;
const MAX_LENS_REPORTS: usize = 16;
const MAX_CONTRADICTIONS: usize = 8;
const MAX_SKIPPED_LENSES: usize = 16;
const MAX_GAPS: usize = 12;
const MAX_DESCRIPTION_CHARS: usize = 512;
const MAX_LOOK_FOR_CHARS: usize = 512;
const MAX_EXPECTATION_CHARS: usize = 256;
const OUTPUT_CONTRACT_FINDING_LIST: &str = "finding-list";
const OUTPUT_CONTRACT_VERDICT_MARKDOWN: &str = "verdict-markdown";
const OUTPUT_CONTRACT_PROPERTY_TABLE: &str = "property-table";
const UNSAFE_CONTEXT_MARKERS: &[&str] = &[
    "raw prompt",
    "provider payload",
    "credential",
    "secret",
    "token",
    "/home/",
    "/tmp/",
    "c:\\",
    "\\users\\",
    "raw diff",
    "raw file body",
    "raw tool output",
    "full transcript",
    "unbounded transcript",
    "unbounded source",
];

const _: () = assert!(MAX_FOCUS_ITEMS > 0);
const _: () = assert!(MAX_REVIEW_CRITERIA > 0);
const _: () = assert!(MAX_CONTEXT_REFS > 0);
const _: () = assert!(MAX_EVIDENCE_HASHES > 0);
const _: () = assert!(MAX_ASSUMPTIONS > 0);
const _: () = assert!(MAX_LENS_REPORTS >= MAX_FOCUS_ITEMS);
const _: () = assert!(MAX_CONTRADICTIONS > 0);
const _: () = assert!(MAX_SKIPPED_LENSES >= MAX_FOCUS_ITEMS);
const _: () = assert!(MAX_GAPS > 0);
const _: () = assert!(MAX_DESCRIPTION_CHARS >= validation::MIN_ADAPTER_FIELD_CHARS);
const _: () = assert!(MAX_LOOK_FOR_CHARS >= validation::MIN_ADAPTER_FIELD_CHARS);
const _: () = assert!(MAX_EXPECTATION_CHARS >= validation::MIN_ADAPTER_FIELD_CHARS);

/// Descriptor-side effects that multi-lens deck logic explicitly does not perform.
pub const PROHIBITED_MULTI_LENS_EFFECTS: &[&str] = &[
    "subagent spawn",
    "model call",
    "process creation",
    "repository read",
    "network access",
    "parallelism enforcement",
    "prompt body generation",
];

/// Execution mode requested by a host for a focus deck.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LensExecutionMode {
    /// Run focus items through host-injected worker services.
    ParallelWorkers,
    /// Render task packets for a host-owned sequential checklist.
    SequentialChecklist,
    /// Delegate to a host-owned external analysis service.
    ExternalService,
}

/// Availability of host-owned lens execution services.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LensWorkerServiceStatus {
    /// Required execution service is injected.
    Available,
    /// Required execution service is missing.
    Missing,
    /// Required execution service is injected but currently unavailable.
    Unavailable,
}

/// Execution planning result status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LensExecutionPlanStatus {
    /// Host may dispatch the typed task packets.
    Ready,
    /// Sequential checklist packets are available without worker dispatch.
    SequentialReady,
    /// A required host service is missing or unavailable.
    MissingService,
}

/// Lens report status supplied by the host after execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LensReportStatus {
    /// Lens completed and produced bounded evidence.
    Completed,
    /// Lens was skipped by host policy.
    Skipped,
    /// Lens failed and should not support final claims.
    Failed,
}

/// Contradiction resolution status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LensContradictionStatus {
    /// Contradiction remains unresolved.
    Unresolved,
    /// Contradiction was resolved by cited evidence.
    Resolved,
}

/// Confidence label for reports and synthesis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LensConfidence {
    /// Low confidence.
    Low,
    /// Medium confidence.
    Medium,
    /// High confidence.
    High,
}

/// Diagnostic code for focus deck validation and planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LensDiagnosticCode {
    /// Required field is empty.
    EmptyField,
    /// Field is too long.
    FieldTooLong,
    /// Field contains a NUL byte.
    NulCharacter,
    /// Collection is empty.
    EmptyCollection,
    /// Collection exceeds the bounded contract.
    TooManyItems,
    /// Focus id is duplicated.
    DuplicateFocus,
    /// Review criterion id is duplicated.
    DuplicateCriterion,
    /// Referenced focus id is missing from the deck.
    UnknownFocus,
    /// Evidence hash is malformed.
    InvalidHash,
    /// Output contract is unsupported.
    UnsupportedOutputContract,
    /// Context or report metadata contains unsafe content.
    UnsafeContext,
    /// Deck body is untrusted.
    UntrustedDeck,
    /// Required host execution service is missing.
    MissingService,
    /// Synthesis attempted to hide an unresolved contradiction.
    UnsupportedClaim,
}

/// Safe diagnostic emitted by multi-lens validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LensDiagnostic {
    code: LensDiagnosticCode,
    focus_id: Option<Text>,
    message: Text,
}

/// Draft review criterion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LensReviewCriterionDraft<'a> {
    /// Stable criterion id.
    pub criterion_id: &'a str,
    /// Safe criterion label or summary.
    pub label: &'a str,
    /// Whether this criterion is required for synthesis.
    pub required: bool,
}

/// Validated review criterion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LensReviewCriterion {
    criterion_id: Text,
    label: Text,
    required: bool,
}

/// Draft focus item in a deck.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FocusItemDraft<'a> {
    /// Stable focus id.
    pub focus_id: &'a str,
    /// Human-readable focus description.
    pub description: &'a str,
    /// Bounded look-for criteria.
    pub look_for: &'a str,
    /// Host-defined retrieval budget reference.
    pub retrieval_budget_ref: &'a str,
    /// Bounded evidence expectation description.
    pub evidence_expectation: &'a str,
    /// Output contract label.
    pub output_contract: &'a str,
}

/// Validated focus item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusItem {
    focus_id: Text,
    description: Text,
    look_for: Text,
    retrieval_budget_ref: Text,
    evidence_expectation: Text,
    output_contract: Text,
}

/// Draft focus deck.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FocusDeckDraft<'a> {
    /// Stable deck id.
    pub deck_id: &'a str,
    /// Host-defined deck family label.
    pub family: &'a str,
    /// Focus items.
    pub items: &'a [FocusItemDraft<'a>],
    /// Review criteria.
    pub review_criteria: &'a [LensReviewCriterionDraft<'a>],
    /// Whether the host trusts this deck body.
    pub trusted: bool,
}

/// Validated focus deck.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusDeck {
    deck_id: Text,
    family: Text,
    items: Collection<FocusItem>,
    review_criteria: Collection<LensReviewCriterion>,
}

/// Draft bounded context reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LensContextRefDraft<'a> {
    /// Safe context label.
    pub label: &'a str,
    /// BLAKE3 evidence identity.
    pub evidence_hash: &'a str,
    /// Whether the context was admitted by the host.
    pub trusted: bool,
}

/// Validated bounded context reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LensContextRef {
    label: Text,
    evidence_hash: Text,
}

/// Draft lens task packet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LensTaskPacketDraft<'a> {
    /// Deck id.
    pub deck_id: &'a str,
    /// Focus id in the deck.
    pub focus_id: &'a str,
    /// Bounded context references.
    pub context_refs: &'a [LensContextRefDraft<'a>],
    /// Retrieval budget reference copied from the deck or narrowed by host.
    pub retrieval_budget_ref: &'a str,
    /// Output contract expected from the worker.
    pub output_contract: &'a str,
}

/// Validated lens task packet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LensTaskPacket {
    deck_id: Text,
    focus_id: Text,
    context_refs: Collection<LensContextRef>,
    retrieval_budget_ref: Text,
    output_contract: Text,
}

/// Execution plan with task packets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LensExecutionPlan {
    status: LensExecutionPlanStatus,
    mode: LensExecutionMode,
    tasks: Collection<LensTaskPacket>,
    diagnostic: Option<LensDiagnostic>,
}

/// Draft lens report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LensReportDraft<'a> {
    /// Report id.
    pub report_id: &'a str,
    /// Focus id this report covers.
    pub focus_id: &'a str,
    /// Report status.
    pub status: LensReportStatus,
    /// BLAKE3 summary identity.
    pub summary_hash: &'a str,
    /// Evidence identities cited by this lens.
    pub evidence_hashes: &'a [&'a str],
    /// Bounded assumption labels.
    pub assumptions: &'a [&'a str],
    /// Confidence label.
    pub confidence: LensConfidence,
}

/// Validated lens report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LensReport {
    report_id: Text,
    focus_id: Text,
    status: LensReportStatus,
    summary_hash: Text,
    evidence_hashes: Collection<Text>,
    assumptions: Collection<Text>,
    confidence: LensConfidence,
}

/// Draft contradiction between lens reports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LensContradictionDraft<'a> {
    /// Contradiction id.
    pub contradiction_id: &'a str,
    /// Lens focus ids involved in the contradiction.
    pub focus_ids: &'a [&'a str],
    /// Evidence identities relevant to the contradiction.
    pub evidence_hashes: &'a [&'a str],
    /// Resolution status.
    pub status: LensContradictionStatus,
    /// Safe summary identity for the contradiction note.
    pub summary_hash: &'a str,
}

/// Validated contradiction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LensContradiction {
    contradiction_id: Text,
    focus_ids: Collection<Text>,
    evidence_hashes: Collection<Text>,
    status: LensContradictionStatus,
    summary_hash: Text,
}

/// Draft synthesis report supplied by host or built from reducers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LensSynthesisDraft<'a> {
    /// Deck id.
    pub deck_id: &'a str,
    /// Validated lens reports.
    pub reports: &'a [LensReportDraft<'a>],
    /// Contradictions.
    pub contradictions: &'a [LensContradictionDraft<'a>],
    /// Skipped focus ids.
    pub skipped_focus_ids: &'a [&'a str],
    /// Unresolved evidence gap labels or identities.
    pub unresolved_gaps: &'a [&'a str],
    /// Overall confidence.
    pub confidence: LensConfidence,
}

/// Validated synthesis report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LensSynthesisReport {
    deck_id: Text,
    reports: Collection<LensReport>,
    contradictions: Collection<LensContradiction>,
    skipped_focus_ids: Collection<Text>,
    unresolved_gaps: Collection<Text>,
    confidence: LensConfidence,
    supports_final_claim: bool,
}

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;
type LensResult<T> = Result<T, LensDiagnostic>;

impl LensDiagnostic {
    /// Diagnostic code.
    #[must_use]
    pub const fn code(&self) -> LensDiagnosticCode {
        self.code
    }

    /// Focus id when available.
    #[must_use]
    pub fn focus_id(&self) -> Option<&str> {
        self.focus_id.as_deref()
    }

    /// Safe diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl LensReviewCriterion {
    /// Criterion id.
    #[must_use]
    pub fn criterion_id(&self) -> &str {
        &self.criterion_id
    }

    /// Criterion label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Whether the criterion is required.
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }
}

impl FocusItem {
    /// Focus id.
    #[must_use]
    pub fn focus_id(&self) -> &str {
        &self.focus_id
    }

    /// Focus description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Look-for criteria.
    #[must_use]
    pub fn look_for(&self) -> &str {
        &self.look_for
    }

    /// Retrieval budget reference.
    #[must_use]
    pub fn retrieval_budget_ref(&self) -> &str {
        &self.retrieval_budget_ref
    }

    /// Evidence expectation.
    #[must_use]
    pub fn evidence_expectation(&self) -> &str {
        &self.evidence_expectation
    }

    /// Output contract.
    #[must_use]
    pub fn output_contract(&self) -> &str {
        &self.output_contract
    }
}

impl FocusDeck {
    /// Deck id.
    #[must_use]
    pub fn deck_id(&self) -> &str {
        &self.deck_id
    }

    /// Deck family.
    #[must_use]
    pub fn family(&self) -> &str {
        &self.family
    }

    /// Focus items.
    #[must_use]
    pub fn items(&self) -> &[FocusItem] {
        &self.items
    }

    /// Review criteria.
    #[must_use]
    pub fn review_criteria(&self) -> &[LensReviewCriterion] {
        &self.review_criteria
    }
}

impl LensContextRef {
    /// Context label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Evidence hash.
    #[must_use]
    pub fn evidence_hash(&self) -> &str {
        &self.evidence_hash
    }
}

impl LensTaskPacket {
    /// Deck id.
    #[must_use]
    pub fn deck_id(&self) -> &str {
        &self.deck_id
    }

    /// Focus id.
    #[must_use]
    pub fn focus_id(&self) -> &str {
        &self.focus_id
    }

    /// Bounded context refs.
    #[must_use]
    pub fn context_refs(&self) -> &[LensContextRef] {
        &self.context_refs
    }

    /// Retrieval budget reference.
    #[must_use]
    pub fn retrieval_budget_ref(&self) -> &str {
        &self.retrieval_budget_ref
    }

    /// Output contract.
    #[must_use]
    pub fn output_contract(&self) -> &str {
        &self.output_contract
    }
}

impl LensExecutionPlan {
    /// Plan status.
    #[must_use]
    pub const fn status(&self) -> LensExecutionPlanStatus {
        self.status
    }

    /// Requested mode.
    #[must_use]
    pub const fn mode(&self) -> LensExecutionMode {
        self.mode
    }

    /// Task packets.
    #[must_use]
    pub fn tasks(&self) -> &[LensTaskPacket] {
        &self.tasks
    }

    /// Diagnostic when planning failed.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&LensDiagnostic> {
        self.diagnostic.as_ref()
    }
}

impl LensReport {
    /// Report id.
    #[must_use]
    pub fn report_id(&self) -> &str {
        &self.report_id
    }

    /// Focus id.
    #[must_use]
    pub fn focus_id(&self) -> &str {
        &self.focus_id
    }

    /// Report status.
    #[must_use]
    pub const fn status(&self) -> LensReportStatus {
        self.status
    }

    /// Summary hash.
    #[must_use]
    pub fn summary_hash(&self) -> &str {
        &self.summary_hash
    }

    /// Evidence hashes.
    #[must_use]
    pub fn evidence_hashes(&self) -> &[Text] {
        &self.evidence_hashes
    }

    /// Assumption labels.
    #[must_use]
    pub fn assumptions(&self) -> &[Text] {
        &self.assumptions
    }

    /// Confidence.
    #[must_use]
    pub const fn confidence(&self) -> LensConfidence {
        self.confidence
    }
}

impl LensContradiction {
    /// Contradiction id.
    #[must_use]
    pub fn contradiction_id(&self) -> &str {
        &self.contradiction_id
    }

    /// Focus ids involved.
    #[must_use]
    pub fn focus_ids(&self) -> &[Text] {
        &self.focus_ids
    }

    /// Evidence hashes involved.
    #[must_use]
    pub fn evidence_hashes(&self) -> &[Text] {
        &self.evidence_hashes
    }

    /// Resolution status.
    #[must_use]
    pub const fn status(&self) -> LensContradictionStatus {
        self.status
    }

    /// Summary hash.
    #[must_use]
    pub fn summary_hash(&self) -> &str {
        &self.summary_hash
    }
}

impl LensSynthesisReport {
    /// Deck id.
    #[must_use]
    pub fn deck_id(&self) -> &str {
        &self.deck_id
    }

    /// Lens reports.
    #[must_use]
    pub fn reports(&self) -> &[LensReport] {
        &self.reports
    }

    /// Contradictions.
    #[must_use]
    pub fn contradictions(&self) -> &[LensContradiction] {
        &self.contradictions
    }

    /// Skipped focus ids.
    #[must_use]
    pub fn skipped_focus_ids(&self) -> &[Text] {
        &self.skipped_focus_ids
    }

    /// Unresolved evidence gaps.
    #[must_use]
    pub fn unresolved_gaps(&self) -> &[Text] {
        &self.unresolved_gaps
    }

    /// Overall confidence.
    #[must_use]
    pub const fn confidence(&self) -> LensConfidence {
        self.confidence
    }

    /// Whether the synthesis supports a final claim.
    #[must_use]
    pub const fn supports_final_claim(&self) -> bool {
        self.supports_final_claim
    }
}

/// Validates a host-defined focus deck.
///
/// # Errors
///
/// Returns [`LensDiagnostic`] when the deck is untrusted, contains duplicate
/// focus IDs, unsafe text, unsupported output contracts, or unbounded lists.
pub fn validate_focus_deck(draft: &FocusDeckDraft<'_>) -> LensResult<FocusDeck> {
    if !draft.trusted {
        return Err(diagnostic(
            LensDiagnosticCode::UntrustedDeck,
            None,
            "focus deck body is not trusted",
        ));
    }
    validate_non_empty_bound("focus_items", draft.items.len(), MAX_FOCUS_ITEMS)?;
    validate_non_empty_bound(
        "lens_review_criteria",
        draft.review_criteria.len(),
        MAX_REVIEW_CRITERIA,
    )?;
    let deck_id = safe_text(DECK_ID_FIELD, draft.deck_id, None)?;
    let family = safe_text(DECK_FAMILY_FIELD, draft.family, None)?;
    let mut items = Collection::with_capacity(draft.items.len());
    for item in draft.items {
        let validated = validate_focus_item(item)?;
        if items
            .iter()
            .any(|existing: &FocusItem| existing.focus_id() == validated.focus_id())
        {
            return Err(diagnostic(
                LensDiagnosticCode::DuplicateFocus,
                Some(validated.focus_id()),
                "duplicate focus id",
            ));
        }
        items.push(validated);
    }

    let mut criteria = Collection::with_capacity(draft.review_criteria.len());
    for criterion in draft.review_criteria {
        let validated = validate_review_criterion(criterion)?;
        if criteria.iter().any(|existing: &LensReviewCriterion| {
            existing.criterion_id() == validated.criterion_id()
        }) {
            return Err(diagnostic(
                LensDiagnosticCode::DuplicateCriterion,
                None,
                "duplicate review criterion id",
            ));
        }
        criteria.push(validated);
    }

    Ok(FocusDeck {
        deck_id,
        family,
        items,
        review_criteria: criteria,
    })
}

/// Validates a lens task packet against a deck.
///
/// # Errors
///
/// Returns [`LensDiagnostic`] for unknown focus IDs, unsafe context labels,
/// missing evidence hashes, unsupported output contracts, or untrusted context.
pub fn validate_lens_task_packet(
    deck: &FocusDeck,
    draft: &LensTaskPacketDraft<'_>,
) -> LensResult<LensTaskPacket> {
    let deck_id = safe_text(DECK_ID_FIELD, draft.deck_id, None)?;
    if deck_id != deck.deck_id() {
        return Err(diagnostic(
            LensDiagnosticCode::UnknownFocus,
            None,
            "task packet deck id does not match deck",
        ));
    }
    let focus_id = safe_text(FOCUS_ID_FIELD, draft.focus_id, None)?;
    let focus_item = deck.items().iter().find(|item| item.focus_id() == focus_id);
    let Some(focus_item) = focus_item else {
        return Err(diagnostic(
            LensDiagnosticCode::UnknownFocus,
            Some(focus_id.as_str()),
            "task packet focus id is not in deck",
        ));
    };
    validate_non_empty_bound(
        "lens_context_refs",
        draft.context_refs.len(),
        MAX_CONTEXT_REFS,
    )?;
    let retrieval_budget_ref = safe_text(RETRIEVAL_BUDGET_FIELD, draft.retrieval_budget_ref, None)?;
    if retrieval_budget_ref != focus_item.retrieval_budget_ref() {
        return Err(diagnostic(
            LensDiagnosticCode::UnknownFocus,
            Some(focus_id.as_str()),
            "retrieval budget reference does not match focus item",
        ));
    }
    let output_contract = safe_output_contract(draft.output_contract)?;
    if output_contract != focus_item.output_contract() {
        return Err(diagnostic(
            LensDiagnosticCode::UnsupportedOutputContract,
            Some(focus_id.as_str()),
            "task output contract does not match focus item",
        ));
    }
    let mut context_refs = Collection::with_capacity(draft.context_refs.len());
    for context in draft.context_refs {
        context_refs.push(validate_context_ref(context, focus_id.as_str())?);
    }

    Ok(LensTaskPacket {
        deck_id,
        focus_id,
        context_refs,
        retrieval_budget_ref,
        output_contract,
    })
}

/// Plans host-owned lens execution without dispatching workers.
#[must_use]
pub fn plan_lens_execution(
    deck: &FocusDeck,
    mode: LensExecutionMode,
    service_status: LensWorkerServiceStatus,
) -> LensExecutionPlan {
    let needs_service = mode != LensExecutionMode::SequentialChecklist;
    if needs_service && service_status != LensWorkerServiceStatus::Available {
        return LensExecutionPlan {
            status: LensExecutionPlanStatus::MissingService,
            mode,
            tasks: Collection::new(),
            diagnostic: Some(diagnostic(
                LensDiagnosticCode::MissingService,
                None,
                "host lens execution service is missing",
            )),
        };
    }

    let mut tasks = Collection::with_capacity(deck.items().len());
    for item in deck.items() {
        tasks.push(LensTaskPacket {
            deck_id: Text::from(deck.deck_id()),
            focus_id: Text::from(item.focus_id()),
            context_refs: Collection::new(),
            retrieval_budget_ref: Text::from(item.retrieval_budget_ref()),
            output_contract: Text::from(item.output_contract()),
        });
    }
    let status = if mode == LensExecutionMode::SequentialChecklist {
        LensExecutionPlanStatus::SequentialReady
    } else {
        LensExecutionPlanStatus::Ready
    };
    LensExecutionPlan {
        status,
        mode,
        tasks,
        diagnostic: None,
    }
}

/// Validates a bounded lens report.
///
/// # Errors
///
/// Returns [`LensDiagnostic`] for unknown focus IDs, unsafe assumptions, malformed
/// evidence hashes, or unbounded evidence lists.
pub fn validate_lens_report(
    deck: &FocusDeck,
    draft: &LensReportDraft<'_>,
) -> LensResult<LensReport> {
    let report_id = safe_text(REPORT_ID_FIELD, draft.report_id, None)?;
    let focus_id = safe_text(FOCUS_ID_FIELD, draft.focus_id, None)?;
    if !deck.items().iter().any(|item| item.focus_id() == focus_id) {
        return Err(diagnostic(
            LensDiagnosticCode::UnknownFocus,
            Some(focus_id.as_str()),
            "lens report focus id is not in deck",
        ));
    }
    let summary_hash = safe_hash(SUMMARY_HASH_FIELD, draft.summary_hash)?;
    let evidence_hashes = validate_hashes("lens_evidence_hashes", draft.evidence_hashes)?;
    validate_collection_limit("lens_assumptions", draft.assumptions.len(), MAX_ASSUMPTIONS)?;
    let mut assumptions = Collection::with_capacity(draft.assumptions.len());
    for assumption in draft.assumptions {
        assumptions.push(safe_text(
            ASSUMPTION_FIELD,
            assumption,
            Some(focus_id.as_str()),
        )?);
    }
    Ok(LensReport {
        report_id,
        focus_id,
        status: draft.status,
        summary_hash,
        evidence_hashes,
        assumptions,
        confidence: draft.confidence,
    })
}

/// Validates and reduces lens synthesis while preserving provenance.
///
/// # Errors
///
/// Returns [`LensDiagnostic`] for malformed report/contradiction/gap metadata or
/// if an unresolved contradiction is paired with a high-confidence final claim.
pub fn validate_lens_synthesis(
    deck: &FocusDeck,
    draft: &LensSynthesisDraft<'_>,
) -> LensResult<LensSynthesisReport> {
    let deck_id = safe_text(DECK_ID_FIELD, draft.deck_id, None)?;
    if deck_id != deck.deck_id() {
        return Err(diagnostic(
            LensDiagnosticCode::UnknownFocus,
            None,
            "synthesis deck id does not match deck",
        ));
    }
    validate_non_empty_bound("lens_reports", draft.reports.len(), MAX_LENS_REPORTS)?;
    validate_collection_limit(
        "lens_contradictions",
        draft.contradictions.len(),
        MAX_CONTRADICTIONS,
    )?;
    validate_collection_limit(
        "lens_skipped_focus_ids",
        draft.skipped_focus_ids.len(),
        MAX_SKIPPED_LENSES,
    )?;
    validate_collection_limit(
        "lens_unresolved_gaps",
        draft.unresolved_gaps.len(),
        MAX_GAPS,
    )?;

    let mut reports = Collection::with_capacity(draft.reports.len());
    for report in draft.reports {
        reports.push(validate_lens_report(deck, report)?);
    }
    let mut contradictions = Collection::with_capacity(draft.contradictions.len());
    for contradiction in draft.contradictions {
        contradictions.push(validate_contradiction(deck, contradiction)?);
    }
    let mut skipped_focus_ids = Collection::with_capacity(draft.skipped_focus_ids.len());
    for focus_id in draft.skipped_focus_ids {
        let safe_focus = safe_text(FOCUS_ID_FIELD, focus_id, None)?;
        if !deck
            .items()
            .iter()
            .any(|item| item.focus_id() == safe_focus)
        {
            return Err(diagnostic(
                LensDiagnosticCode::UnknownFocus,
                Some(safe_focus.as_str()),
                "skipped focus id is not in deck",
            ));
        }
        skipped_focus_ids.push(safe_focus);
    }
    let mut unresolved_gaps = Collection::with_capacity(draft.unresolved_gaps.len());
    for gap in draft.unresolved_gaps {
        unresolved_gaps.push(safe_text(GAP_ID_FIELD, gap, None)?);
    }
    let unresolved_contradiction = contradictions
        .iter()
        .any(|item| item.status() == LensContradictionStatus::Unresolved);
    let supports_final_claim = !unresolved_contradiction && unresolved_gaps.is_empty();
    if unresolved_contradiction && draft.confidence == LensConfidence::High {
        return Err(diagnostic(
            LensDiagnosticCode::UnsupportedClaim,
            None,
            "high-confidence synthesis cannot hide unresolved contradictions",
        ));
    }

    Ok(LensSynthesisReport {
        deck_id,
        reports,
        contradictions,
        skipped_focus_ids,
        unresolved_gaps,
        confidence: draft.confidence,
        supports_final_claim,
    })
}

fn validate_focus_item(draft: &FocusItemDraft<'_>) -> LensResult<FocusItem> {
    let focus_id = safe_text(FOCUS_ID_FIELD, draft.focus_id, None)?;
    let description = safe_bounded_text(
        FOCUS_DESCRIPTION_FIELD,
        draft.description,
        MAX_DESCRIPTION_CHARS,
        Some(focus_id.as_str()),
    )?;
    let look_for = safe_bounded_text(
        LOOK_FOR_FIELD,
        draft.look_for,
        MAX_LOOK_FOR_CHARS,
        Some(focus_id.as_str()),
    )?;
    let retrieval_budget_ref = safe_text(
        RETRIEVAL_BUDGET_FIELD,
        draft.retrieval_budget_ref,
        Some(focus_id.as_str()),
    )?;
    let evidence_expectation = safe_bounded_text(
        EVIDENCE_EXPECTATION_FIELD,
        draft.evidence_expectation,
        MAX_EXPECTATION_CHARS,
        Some(focus_id.as_str()),
    )?;
    let output_contract = safe_output_contract(draft.output_contract)?;
    Ok(FocusItem {
        focus_id,
        description,
        look_for,
        retrieval_budget_ref,
        evidence_expectation,
        output_contract,
    })
}

fn validate_review_criterion(
    draft: &LensReviewCriterionDraft<'_>,
) -> LensResult<LensReviewCriterion> {
    let criterion_id = safe_text(CRITERION_ID_FIELD, draft.criterion_id, None)?;
    let label = safe_text(LOOK_FOR_FIELD, draft.label, None)?;
    Ok(LensReviewCriterion {
        criterion_id,
        label,
        required: draft.required,
    })
}

fn validate_context_ref(
    draft: &LensContextRefDraft<'_>,
    focus_id: &str,
) -> LensResult<LensContextRef> {
    if !draft.trusted {
        return Err(diagnostic(
            LensDiagnosticCode::UnsafeContext,
            Some(focus_id),
            "untrusted context reference",
        ));
    }
    let label = safe_text(CONTEXT_LABEL_FIELD, draft.label, Some(focus_id))?;
    let evidence_hash = safe_hash(SUMMARY_HASH_FIELD, draft.evidence_hash)?;
    Ok(LensContextRef {
        label,
        evidence_hash,
    })
}

fn validate_contradiction(
    deck: &FocusDeck,
    draft: &LensContradictionDraft<'_>,
) -> LensResult<LensContradiction> {
    let contradiction_id = safe_text(CONTRADICTION_ID_FIELD, draft.contradiction_id, None)?;
    validate_non_empty_bound(
        "lens_contradiction_focus_ids",
        draft.focus_ids.len(),
        MAX_FOCUS_ITEMS,
    )?;
    let mut focus_ids = Collection::with_capacity(draft.focus_ids.len());
    for focus_id in draft.focus_ids {
        let safe_focus = safe_text(FOCUS_ID_FIELD, focus_id, None)?;
        if !deck
            .items()
            .iter()
            .any(|item| item.focus_id() == safe_focus)
        {
            return Err(diagnostic(
                LensDiagnosticCode::UnknownFocus,
                Some(safe_focus.as_str()),
                "contradiction focus id is not in deck",
            ));
        }
        focus_ids.push(safe_focus);
    }
    let evidence_hashes = validate_hashes("lens_contradiction_hashes", draft.evidence_hashes)?;
    let summary_hash = safe_hash(SUMMARY_HASH_FIELD, draft.summary_hash)?;
    Ok(LensContradiction {
        contradiction_id,
        focus_ids,
        evidence_hashes,
        status: draft.status,
        summary_hash,
    })
}

fn validate_hashes(field: &'static str, hashes: &[&str]) -> LensResult<Collection<Text>> {
    validate_non_empty_bound(field, hashes.len(), MAX_EVIDENCE_HASHES)?;
    let mut values = Collection::with_capacity(hashes.len());
    for hash in hashes {
        values.push(safe_hash(SUMMARY_HASH_FIELD, hash)?);
    }
    Ok(values)
}

fn safe_output_contract(value: &str) -> LensResult<Text> {
    let contract = safe_text(OUTPUT_CONTRACT_FIELD, value, None)?;
    match contract.as_str() {
        OUTPUT_CONTRACT_FINDING_LIST
        | OUTPUT_CONTRACT_VERDICT_MARKDOWN
        | OUTPUT_CONTRACT_PROPERTY_TABLE => Ok(contract),
        _ => Err(diagnostic(
            LensDiagnosticCode::UnsupportedOutputContract,
            None,
            "unsupported output contract",
        )),
    }
}

fn safe_bounded_text(
    field: FieldName,
    value: &str,
    max_chars: usize,
    focus_id: Option<&str>,
) -> LensResult<Text> {
    let text = safe_text(field, value, focus_id)?;
    let actual_chars = text.chars().count();
    if actual_chars > max_chars {
        return Err(diagnostic(
            LensDiagnosticCode::FieldTooLong,
            focus_id,
            field.as_str(),
        ));
    }
    Ok(text)
}

fn safe_text(field: FieldName, value: &str, focus_id: Option<&str>) -> LensResult<Text> {
    let normalized = validation::normalized_adapter_field(field, value)
        .map_err(|error| map_adapter_error(&error))?;
    let lower = normalized.to_ascii_lowercase();
    if UNSAFE_CONTEXT_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Err(diagnostic(
            LensDiagnosticCode::UnsafeContext,
            focus_id,
            "lens metadata contains unsafe context marker",
        ));
    }
    Ok(normalized)
}

fn safe_hash(field: FieldName, value: &str) -> LensResult<Text> {
    let normalized = validation::normalized_adapter_field(field, value)
        .map_err(|error| map_adapter_error(&error))?;
    if Hash::new(&normalized).is_ok() {
        Ok(normalized)
    } else {
        Err(diagnostic(
            LensDiagnosticCode::InvalidHash,
            None,
            field.as_str(),
        ))
    }
}

fn validate_non_empty_bound(field: &'static str, len: usize, max: usize) -> LensResult<()> {
    if len == 0 {
        return Err(LensDiagnostic {
            code: LensDiagnosticCode::EmptyCollection,
            focus_id: None,
            message: Text::from(field),
        });
    }
    validate_collection_limit(field, len, max)
}

fn validate_collection_limit(field: &'static str, len: usize, max: usize) -> LensResult<()> {
    if len > max {
        return Err(LensDiagnostic {
            code: LensDiagnosticCode::TooManyItems,
            focus_id: None,
            message: Text::from(field),
        });
    }
    Ok(())
}

fn map_adapter_error(error: &AdapterContractError) -> LensDiagnostic {
    match error {
        AdapterContractError::EmptyField { field } => {
            diagnostic(LensDiagnosticCode::EmptyField, None, field)
        }
        AdapterContractError::FieldTooLong { field, .. } => {
            diagnostic(LensDiagnosticCode::FieldTooLong, None, field)
        }
        AdapterContractError::NulCharacter { field, .. } => {
            diagnostic(LensDiagnosticCode::NulCharacter, None, field)
        }
        AdapterContractError::InvalidEvidenceHash { field, .. } => {
            diagnostic(LensDiagnosticCode::InvalidHash, None, field)
        }
        AdapterContractError::EmptyCollection { field } => {
            diagnostic(LensDiagnosticCode::EmptyCollection, None, field)
        }
        AdapterContractError::UnsupportedStatusState { field, .. }
        | AdapterContractError::UnsupportedStatusObservationKind { field, .. }
        | AdapterContractError::UnsupportedSessionReferenceKind { field, .. }
        | AdapterContractError::UnsupportedWorkspaceValue { field, .. }
        | AdapterContractError::UnsupportedDataspaceValue { field, .. }
        | AdapterContractError::UnsupportedRouterValue { field, .. } => {
            diagnostic(LensDiagnosticCode::UnsafeContext, None, field)
        }
    }
}

fn diagnostic(code: LensDiagnosticCode, focus_id: Option<&str>, message: &str) -> LensDiagnostic {
    LensDiagnostic {
        code,
        focus_id: focus_id.map(Text::from),
        message: Text::from(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DECK_ID: &str = "security-review";
    const DECK_FAMILY: &str = "host-defined-security";
    const FOCUS_BOUNDARY: &str = "host-owned-boundary";
    const FOCUS_EVIDENCE: &str = "evidence-shape";
    const DESCRIPTION: &str = "Check whether unsafe operations remain host-owned.";
    const LOOK_FOR: &str = "host-owned service claims and non-claims";
    const BUDGET_REF: &str = "budget-focus-small";
    const EVIDENCE_EXPECTATION: &str = "bounded source labels and BLAKE3 evidence";
    const CRITERION_ID: &str = "preserve-provenance";
    const CRITERION_LABEL: &str = "Preserve lens ids and evidence hashes.";
    const CONTEXT_LABEL: &str = "src/adapters-summary";
    const REPORT_ID: &str = "report-boundary";
    const CONTRADICTION_ID: &str = "contradiction-boundary";
    const GAP_ID: &str = "missing-host-policy";
    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const UNSAFE_CONTEXT: &str = "full transcript provider payload";
    static REVIEW_CRITERIA: [LensReviewCriterionDraft<'static>; 1] = [LensReviewCriterionDraft {
        criterion_id: CRITERION_ID,
        label: CRITERION_LABEL,
        required: true,
    }];

    fn focus(focus_id: &'static str) -> FocusItemDraft<'static> {
        FocusItemDraft {
            focus_id,
            description: DESCRIPTION,
            look_for: LOOK_FOR,
            retrieval_budget_ref: BUDGET_REF,
            evidence_expectation: EVIDENCE_EXPECTATION,
            output_contract: OUTPUT_CONTRACT_FINDING_LIST,
        }
    }

    fn deck_draft<'a>(items: &'a [FocusItemDraft<'a>]) -> FocusDeckDraft<'a> {
        FocusDeckDraft {
            deck_id: DECK_ID,
            family: DECK_FAMILY,
            items,
            review_criteria: &REVIEW_CRITERIA,
            trusted: true,
        }
    }

    fn deck() -> FocusDeck {
        validate_focus_deck(&deck_draft(&[focus(FOCUS_BOUNDARY), focus(FOCUS_EVIDENCE)])).unwrap()
    }

    fn context_ref() -> LensContextRefDraft<'static> {
        LensContextRefDraft {
            label: CONTEXT_LABEL,
            evidence_hash: HASH_A,
            trusted: true,
        }
    }

    fn task_draft<'a>(contexts: &'a [LensContextRefDraft<'a>]) -> LensTaskPacketDraft<'a> {
        LensTaskPacketDraft {
            deck_id: DECK_ID,
            focus_id: FOCUS_BOUNDARY,
            context_refs: contexts,
            retrieval_budget_ref: BUDGET_REF,
            output_contract: OUTPUT_CONTRACT_FINDING_LIST,
        }
    }

    fn report_draft(focus_id: &'static str) -> LensReportDraft<'static> {
        LensReportDraft {
            report_id: REPORT_ID,
            focus_id,
            status: LensReportStatus::Completed,
            summary_hash: HASH_A,
            evidence_hashes: &[HASH_A],
            assumptions: &["host supplied bounded context"],
            confidence: LensConfidence::Medium,
        }
    }

    #[test]
    fn host_defined_focus_deck_validates_without_hermes_defaults() {
        // r[verify agent_harness_runtime.hermes_multi_lens.focus_decks]
        // r[verify agent_harness_runtime.hermes_multi_lens.validation]
        let deck = deck();

        assert_eq!(deck.deck_id(), DECK_ID);
        assert_eq!(deck.family(), DECK_FAMILY);
        assert_eq!(deck.items().len(), 2);
        assert_eq!(deck.review_criteria().len(), 1);
        assert!(PROHIBITED_MULTI_LENS_EFFECTS.contains(&"subagent spawn"));
    }

    #[test]
    fn lens_task_packets_reject_unsafe_context_and_unknown_focus() {
        // r[verify agent_harness_runtime.hermes_multi_lens.validation]
        let deck = deck();
        let contexts = [context_ref()];
        let valid_task = validate_lens_task_packet(&deck, &task_draft(&contexts)).unwrap();
        let unsafe_contexts = [LensContextRefDraft {
            label: UNSAFE_CONTEXT,
            ..context_ref()
        }];
        let unsafe_error = validate_lens_task_packet(&deck, &task_draft(&unsafe_contexts))
            .expect_err("unsafe context must fail");
        let unknown_error = validate_lens_task_packet(
            &deck,
            &LensTaskPacketDraft {
                focus_id: "missing-focus",
                ..task_draft(&contexts)
            },
        )
        .expect_err("unknown focus must fail");

        assert_eq!(valid_task.context_refs().len(), 1);
        assert_eq!(unsafe_error.code(), LensDiagnosticCode::UnsafeContext);
        assert_eq!(unknown_error.code(), LensDiagnosticCode::UnknownFocus);
    }

    #[test]
    fn duplicate_focus_and_untrusted_decks_fail_validation() {
        // r[verify agent_harness_runtime.hermes_multi_lens.verification]
        let duplicate_items = [focus(FOCUS_BOUNDARY), focus(FOCUS_BOUNDARY)];
        let duplicate_error = validate_focus_deck(&deck_draft(&duplicate_items))
            .expect_err("duplicate focus ids must fail");
        let untrusted_error = validate_focus_deck(&FocusDeckDraft {
            trusted: false,
            ..deck_draft(&[focus(FOCUS_BOUNDARY)])
        })
        .expect_err("untrusted deck must fail");

        assert_eq!(duplicate_error.code(), LensDiagnosticCode::DuplicateFocus);
        assert_eq!(untrusted_error.code(), LensDiagnosticCode::UntrustedDeck);
    }

    #[test]
    fn execution_planning_requires_host_services_for_parallel_modes() {
        // r[verify agent_harness_runtime.hermes_multi_lens.host_owned]
        let deck = deck();
        let parallel_missing = plan_lens_execution(
            &deck,
            LensExecutionMode::ParallelWorkers,
            LensWorkerServiceStatus::Missing,
        );
        let sequential = plan_lens_execution(
            &deck,
            LensExecutionMode::SequentialChecklist,
            LensWorkerServiceStatus::Missing,
        );
        let external_ready = plan_lens_execution(
            &deck,
            LensExecutionMode::ExternalService,
            LensWorkerServiceStatus::Available,
        );

        assert_eq!(
            parallel_missing.status(),
            LensExecutionPlanStatus::MissingService
        );
        assert_eq!(
            sequential.status(),
            LensExecutionPlanStatus::SequentialReady
        );
        assert_eq!(external_ready.status(), LensExecutionPlanStatus::Ready);
        assert_eq!(sequential.tasks().len(), deck.items().len());
    }

    #[test]
    fn synthesis_preserves_unresolved_contradictions_and_skips_final_claim() {
        // r[verify agent_harness_runtime.hermes_multi_lens.synthesis]
        let deck = deck();
        let reports = [
            report_draft(FOCUS_BOUNDARY),
            LensReportDraft {
                report_id: "report-evidence",
                ..report_draft(FOCUS_EVIDENCE)
            },
        ];
        let contradictions = [LensContradictionDraft {
            contradiction_id: CONTRADICTION_ID,
            focus_ids: &[FOCUS_BOUNDARY, FOCUS_EVIDENCE],
            evidence_hashes: &[HASH_A, HASH_B],
            status: LensContradictionStatus::Unresolved,
            summary_hash: HASH_B,
        }];
        let synthesis = validate_lens_synthesis(
            &deck,
            &LensSynthesisDraft {
                deck_id: DECK_ID,
                reports: &reports,
                contradictions: &contradictions,
                skipped_focus_ids: &[FOCUS_EVIDENCE],
                unresolved_gaps: &[GAP_ID],
                confidence: LensConfidence::Medium,
            },
        )
        .unwrap();

        assert_eq!(synthesis.contradictions().len(), 1);
        assert_eq!(synthesis.skipped_focus_ids().len(), 1);
        assert_eq!(synthesis.unresolved_gaps().len(), 1);
        assert!(!synthesis.supports_final_claim());
    }

    #[test]
    fn synthesis_rejects_high_confidence_with_unresolved_contradiction() {
        // r[verify agent_harness_runtime.hermes_multi_lens.verification]
        let deck = deck();
        let reports = [report_draft(FOCUS_BOUNDARY)];
        let contradictions = [LensContradictionDraft {
            contradiction_id: CONTRADICTION_ID,
            focus_ids: &[FOCUS_BOUNDARY],
            evidence_hashes: &[HASH_A],
            status: LensContradictionStatus::Unresolved,
            summary_hash: HASH_B,
        }];
        let error = validate_lens_synthesis(
            &deck,
            &LensSynthesisDraft {
                deck_id: DECK_ID,
                reports: &reports,
                contradictions: &contradictions,
                skipped_focus_ids: &[],
                unresolved_gaps: &[],
                confidence: LensConfidence::High,
            },
        )
        .expect_err("high-confidence unresolved contradiction must fail");

        assert_eq!(error.code(), LensDiagnosticCode::UnsupportedClaim);
    }
}
