//! Optional plan/loop mode finite-state contracts.
//!
//! Hosts may use these descriptors and reducers to build `/plan` or `/loop`-like
//! products, but this module never installs commands, changes tools, calls
//! models, schedules timers, persists sessions, delegates workers, probes
//! providers, or enforces read-only behavior by prompt text.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::assertion_density,
        tigerstyle::non_trait_imports,
        tigerstyle::raw_arithmetic_overflow
    )
)]

use crate::artifact::Hash;
use crate::validation::{self, AdapterContractError, FieldName};

const MODE_ID_FIELD: FieldName = FieldName::new("mode_id");
const MODE_LABEL_FIELD: FieldName = FieldName::new("mode_label");
const CAPABILITY_CEILING_FIELD: FieldName = FieldName::new("mode_capability_ceiling");
const MARKER_FIELD: FieldName = FieldName::new("mode_marker");
const OUTPUT_CONTRACT_FIELD: FieldName = FieldName::new("mode_output_contract");
const STEP_ID_FIELD: FieldName = FieldName::new("plan_step_id");
const STEP_LABEL_FIELD: FieldName = FieldName::new("plan_step_label");
const WORK_ID_FIELD: FieldName = FieldName::new("loop_work_id");
const LOOP_REASON_FIELD: FieldName = FieldName::new("loop_reason");
const RECEIPT_STATUS_FIELD: FieldName = FieldName::new("mode_receipt_status");
const RECEIPT_ISSUE_FIELD: FieldName = FieldName::new("mode_receipt_issue");
const HASH_FIELD: FieldName = FieldName::new("mode_hash");

const MAX_PLAN_STEPS: usize = 16;
const MAX_MARKER_CHARS: usize = 64;
const MAX_MODE_LABEL_CHARS: usize = 128;
const MAX_STEP_LABEL_CHARS: usize = 256;
const MAX_PLAN_BODY_CHARS: usize = 4_096;
const MAX_LOOP_BUDGET_UNITS: u16 = 16;
const MIN_LOOP_BUDGET_UNITS: u16 = 1;
const STALE_ACK_EPOCH_DELTA: u32 = 2;
const INITIAL_STEP_EPOCH: u32 = 0;
const PLAN_STEP_PREFIX: &str = "- [";
const PLAN_STEP_SEPARATOR: &str = "]";
const SELECTED_STEP_CONTRACT: &str = "selected-step-list";
const LOOP_WORK_CONTRACT: &str = "loop-work-item";
const UNSAFE_MODE_MARKERS: &[&str] = &[
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
    "provider probe",
    "install command",
    "unbounded transcript",
];

const _: () = assert!(MAX_PLAN_STEPS > 0);
const _: () = assert!(MAX_MARKER_CHARS >= validation::MIN_ADAPTER_FIELD_CHARS);
const _: () = assert!(MAX_MODE_LABEL_CHARS >= validation::MIN_ADAPTER_FIELD_CHARS);
const _: () = assert!(MAX_STEP_LABEL_CHARS >= validation::MIN_ADAPTER_FIELD_CHARS);
const _: () = assert!(MAX_PLAN_BODY_CHARS >= MAX_STEP_LABEL_CHARS);
const _: () = assert!(MAX_LOOP_BUDGET_UNITS >= MIN_LOOP_BUDGET_UNITS);
const _: () = assert!(STALE_ACK_EPOCH_DELTA > 0);

/// Product controls that plan/loop FSM logic explicitly does not perform.
pub const PROHIBITED_PLAN_LOOP_EFFECTS: &[&str] = &[
    "command registration",
    "keyboard shortcut installation",
    "UI picker opening",
    "tool activation",
    "read-only enforcement",
    "delegation",
    "timer scheduling",
    "provider probing",
    "session persistence",
];

/// Host-declared mode posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModePosture {
    /// Planning guidance; mutation must remain host-forbidden by separate gates.
    PlanningReadOnly,
    /// Execute selected steps through host-owned services.
    SelectedStepExecution,
    /// Continue loop work through typed continuation policy.
    LoopContinuation,
}

/// Mode runtime state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModeRuntimeState {
    /// Mode is not active.
    Inactive,
    /// Mode has extracted plan steps.
    Planning,
    /// Mode has selected work.
    Selected,
    /// Mode has active loop work.
    Looping,
    /// Mode is completed.
    Completed,
}

/// Host controller service status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModeControllerStatus {
    /// Host injected a controller.
    Available,
    /// Host did not inject a controller.
    Missing,
}

/// Host controller planning status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModeControllerPlanStatus {
    /// Descriptor can be controlled by the host.
    Ready,
    /// No host controller is present; no product behavior is installed.
    MissingController,
}

/// Retry classification supplied by host adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModeRetryClass {
    /// Transient host-supplied failure.
    Transient,
    /// Rate limit supplied by host adapter.
    RateLimited,
    /// Permanent failure.
    Permanent,
    /// Unknown classification; host must decide.
    Unknown,
}

/// Plan extraction status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanExtractionStatus {
    /// Explicit markers were found and steps were extracted.
    Extracted,
    /// No configured marker pair was found.
    NoPlan,
}

/// Step selection status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepSelectionStatus {
    /// Step is available but not selected.
    Pending,
    /// Step was selected by host state.
    Selected,
    /// Step was completed by host state.
    Completed,
}

/// Breakout acknowledgement status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BreakoutAckStatus {
    /// Acknowledgement matches active loop work.
    Accepted,
    /// Acknowledgement references another work item.
    WorkMismatch,
    /// Acknowledgement is stale.
    Stale,
}

/// Plan/loop diagnostic code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModeDiagnosticCode {
    /// Required field is empty.
    EmptyField,
    /// Field is too long.
    FieldTooLong,
    /// Field contains a NUL byte.
    NulCharacter,
    /// Metadata includes unsafe content.
    UnsafeText,
    /// Output contract is unsupported.
    UnsupportedOutputContract,
    /// Marker pair is malformed or missing.
    MalformedMarker,
    /// Step id is duplicated.
    DuplicateStep,
    /// Step id is unknown.
    UnknownStep,
    /// Step update or acknowledgement is stale.
    StaleUpdate,
    /// Loop budget is outside bounds.
    InvalidBudget,
    /// Host controller is missing.
    MissingController,
    /// Receipt hash is malformed.
    InvalidHash,
}

/// Safe diagnostic emitted by plan/loop mode contracts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeDiagnostic {
    code: ModeDiagnosticCode,
    subject_id: Option<Text>,
    message: Text,
}

/// Draft mode descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModeDescriptorDraft<'a> {
    /// Stable mode id.
    pub mode_id: &'a str,
    /// Safe host-visible label.
    pub label: &'a str,
    /// Mode posture.
    pub posture: ModePosture,
    /// Capability ceiling label.
    pub capability_ceiling: &'a str,
    /// Explicit plan start marker.
    pub plan_start_marker: &'a str,
    /// Explicit plan end marker.
    pub plan_end_marker: &'a str,
    /// Output contract label.
    pub output_contract: &'a str,
    /// Loop continuation budget units.
    pub loop_budget_units: u16,
    /// Whether the host trusts this descriptor body.
    pub trusted: bool,
}

/// Validated mode descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeDescriptor {
    mode_id: Text,
    label: Text,
    posture: ModePosture,
    capability_ceiling: Text,
    plan_start_marker: Text,
    plan_end_marker: Text,
    output_contract: Text,
    loop_budget_units: u16,
}

/// Extracted plan step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanStep {
    step_id: Text,
    label: Text,
    status: StepSelectionStatus,
    selected_epoch: u32,
}

/// Plan extraction result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanExtraction {
    status: PlanExtractionStatus,
    steps: Collection<PlanStep>,
    diagnostic: Option<ModeDiagnostic>,
}

/// Mode finite-state data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeState {
    mode_id: Text,
    runtime_state: ModeRuntimeState,
    steps: Collection<PlanStep>,
    active_loop: Option<LoopWorkItem>,
    retry_class: ModeRetryClass,
}

/// Selected-step update draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedStepUpdateDraft<'a> {
    /// Step id to select or complete.
    pub step_id: &'a str,
    /// New selection status.
    pub status: StepSelectionStatus,
    /// Host-supplied update epoch.
    pub update_epoch: u32,
}

/// Loop condition draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoopConditionDraft<'a> {
    /// Safe loop work id.
    pub work_id: &'a str,
    /// Bounded reason label.
    pub reason: &'a str,
    /// Host-supplied continuation budget.
    pub budget_units: u16,
    /// Retry classification supplied by host adapter.
    pub retry_class: ModeRetryClass,
}

/// Active loop work item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoopWorkItem {
    work_id: Text,
    reason: Text,
    budget_units: u16,
    retry_class: ModeRetryClass,
}

/// Breakout acknowledgement draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BreakoutAcknowledgementDraft<'a> {
    /// Work id being acknowledged.
    pub work_id: &'a str,
    /// Whether the host reports breakout success.
    pub success: bool,
    /// Epoch from the acknowledgement.
    pub acknowledgement_epoch: u32,
    /// Host current epoch.
    pub current_epoch: u32,
}

/// Breakout acknowledgement result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreakoutAcknowledgement {
    status: BreakoutAckStatus,
    work_id: Option<Text>,
    diagnostic: Option<ModeDiagnostic>,
}

/// Host controller plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeControllerPlan {
    status: ModeControllerPlanStatus,
    mode_id: Text,
    installed_commands: Collection<Text>,
    diagnostic: Option<ModeDiagnostic>,
}

/// Mode receipt draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModeReceiptDraft<'a> {
    /// Mode id.
    pub mode_id: &'a str,
    /// Receipt status label.
    pub status: &'a str,
    /// State hash.
    pub state_hash: &'a str,
    /// Optional safe issue code.
    pub issue_code: Option<&'a str>,
}

/// Validated safe mode receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeReceipt {
    mode_id: Text,
    status: Text,
    state_hash: Text,
    issue_code: Option<Text>,
}

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;
type ModeResult<T> = Result<T, ModeDiagnostic>;

impl ModeDiagnostic {
    /// Diagnostic code.
    #[must_use]
    pub const fn code(&self) -> ModeDiagnosticCode {
        self.code
    }

    /// Optional subject id.
    #[must_use]
    pub fn subject_id(&self) -> Option<&str> {
        self.subject_id.as_deref()
    }

    /// Safe message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl ModeDescriptor {
    /// Mode id.
    #[must_use]
    pub fn mode_id(&self) -> &str {
        &self.mode_id
    }

    /// Mode label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Mode posture.
    #[must_use]
    pub const fn posture(&self) -> ModePosture {
        self.posture
    }

    /// Capability ceiling label.
    #[must_use]
    pub fn capability_ceiling(&self) -> &str {
        &self.capability_ceiling
    }

    /// Plan start marker.
    #[must_use]
    pub fn plan_start_marker(&self) -> &str {
        &self.plan_start_marker
    }

    /// Plan end marker.
    #[must_use]
    pub fn plan_end_marker(&self) -> &str {
        &self.plan_end_marker
    }

    /// Output contract.
    #[must_use]
    pub fn output_contract(&self) -> &str {
        &self.output_contract
    }

    /// Loop budget units.
    #[must_use]
    pub const fn loop_budget_units(&self) -> u16 {
        self.loop_budget_units
    }
}

impl PlanStep {
    /// Step id.
    #[must_use]
    pub fn step_id(&self) -> &str {
        &self.step_id
    }

    /// Step label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Selection status.
    #[must_use]
    pub const fn status(&self) -> StepSelectionStatus {
        self.status
    }

    /// Host selection epoch.
    #[must_use]
    pub const fn selected_epoch(&self) -> u32 {
        self.selected_epoch
    }
}

impl PlanExtraction {
    /// Extraction status.
    #[must_use]
    pub const fn status(&self) -> PlanExtractionStatus {
        self.status
    }

    /// Extracted steps.
    #[must_use]
    pub fn steps(&self) -> &[PlanStep] {
        &self.steps
    }

    /// Diagnostic when no plan or invalid input was found.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&ModeDiagnostic> {
        self.diagnostic.as_ref()
    }
}

impl ModeState {
    /// Mode id.
    #[must_use]
    pub fn mode_id(&self) -> &str {
        &self.mode_id
    }

    /// Runtime state.
    #[must_use]
    pub const fn runtime_state(&self) -> ModeRuntimeState {
        self.runtime_state
    }

    /// Plan steps.
    #[must_use]
    pub fn steps(&self) -> &[PlanStep] {
        &self.steps
    }

    /// Active loop work.
    #[must_use]
    pub fn active_loop(&self) -> Option<&LoopWorkItem> {
        self.active_loop.as_ref()
    }

    /// Retry classification.
    #[must_use]
    pub const fn retry_class(&self) -> ModeRetryClass {
        self.retry_class
    }
}

impl LoopWorkItem {
    /// Work id.
    #[must_use]
    pub fn work_id(&self) -> &str {
        &self.work_id
    }

    /// Reason label.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Budget units.
    #[must_use]
    pub const fn budget_units(&self) -> u16 {
        self.budget_units
    }

    /// Retry classification.
    #[must_use]
    pub const fn retry_class(&self) -> ModeRetryClass {
        self.retry_class
    }
}

impl BreakoutAcknowledgement {
    /// Acknowledgement status.
    #[must_use]
    pub const fn status(&self) -> BreakoutAckStatus {
        self.status
    }

    /// Acknowledged work id when valid.
    #[must_use]
    pub fn work_id(&self) -> Option<&str> {
        self.work_id.as_deref()
    }

    /// Diagnostic when rejected.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&ModeDiagnostic> {
        self.diagnostic.as_ref()
    }
}

impl ModeControllerPlan {
    /// Controller plan status.
    #[must_use]
    pub const fn status(&self) -> ModeControllerPlanStatus {
        self.status
    }

    /// Mode id.
    #[must_use]
    pub fn mode_id(&self) -> &str {
        &self.mode_id
    }

    /// Commands installed by reusable logic. This is always empty.
    #[must_use]
    pub fn installed_commands(&self) -> &[Text] {
        &self.installed_commands
    }

    /// Diagnostic when controller is missing.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&ModeDiagnostic> {
        self.diagnostic.as_ref()
    }
}

impl ModeReceipt {
    /// Mode id.
    #[must_use]
    pub fn mode_id(&self) -> &str {
        &self.mode_id
    }

    /// Receipt status.
    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }

    /// State hash.
    #[must_use]
    pub fn state_hash(&self) -> &str {
        &self.state_hash
    }

    /// Issue code.
    #[must_use]
    pub fn issue_code(&self) -> Option<&str> {
        self.issue_code.as_deref()
    }
}

/// Validates a host-provided mode descriptor.
///
/// # Errors
///
/// Returns [`ModeDiagnostic`] for unsafe text, malformed markers, unsupported
/// output contracts, untrusted descriptors, or invalid loop budgets.
pub fn validate_mode_descriptor(draft: &ModeDescriptorDraft<'_>) -> ModeResult<ModeDescriptor> {
    if !draft.trusted {
        return Err(diagnostic(
            ModeDiagnosticCode::UnsafeText,
            None,
            "untrusted mode descriptor",
        ));
    }
    validate_loop_budget(draft.loop_budget_units)?;
    let mode_id = safe_text(MODE_ID_FIELD, draft.mode_id, None)?;
    let label = safe_bounded_text(MODE_LABEL_FIELD, draft.label, MAX_MODE_LABEL_CHARS, None)?;
    let capability_ceiling = safe_text(CAPABILITY_CEILING_FIELD, draft.capability_ceiling, None)?;
    let plan_start_marker = safe_bounded_text(
        MARKER_FIELD,
        draft.plan_start_marker,
        MAX_MARKER_CHARS,
        Some(mode_id.as_str()),
    )?;
    let plan_end_marker = safe_bounded_text(
        MARKER_FIELD,
        draft.plan_end_marker,
        MAX_MARKER_CHARS,
        Some(mode_id.as_str()),
    )?;
    if plan_start_marker == plan_end_marker {
        return Err(diagnostic(
            ModeDiagnosticCode::MalformedMarker,
            Some(mode_id.as_str()),
            "plan markers must be distinct",
        ));
    }
    let output_contract = safe_output_contract(draft.output_contract)?;
    Ok(ModeDescriptor {
        mode_id,
        label,
        posture: draft.posture,
        capability_ceiling,
        plan_start_marker,
        plan_end_marker,
        output_contract,
        loop_budget_units: draft.loop_budget_units,
    })
}

/// Extracts plan steps only from configured explicit markers.
///
/// # Errors
///
/// Returns [`ModeDiagnostic`] for malformed marked plan bodies, duplicate steps,
/// unsafe step labels, or unbounded text.
pub fn extract_plan_steps(descriptor: &ModeDescriptor, text: &str) -> ModeResult<PlanExtraction> {
    validate_plan_text_size(text)?;
    let Some(start_index) = text.find(descriptor.plan_start_marker()) else {
        return Ok(PlanExtraction {
            status: PlanExtractionStatus::NoPlan,
            steps: Collection::new(),
            diagnostic: Some(diagnostic(
                ModeDiagnosticCode::MalformedMarker,
                Some(descriptor.mode_id()),
                "configured plan start marker not found",
            )),
        });
    };
    let body_start = start_index + descriptor.plan_start_marker().len();
    let Some(end_offset) = text[body_start..].find(descriptor.plan_end_marker()) else {
        return Err(diagnostic(
            ModeDiagnosticCode::MalformedMarker,
            Some(descriptor.mode_id()),
            "configured plan end marker not found",
        ));
    };
    let body_end = body_start + end_offset;
    let body = &text[body_start..body_end];
    let mut steps = Collection::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let step = parse_step_line(trimmed, descriptor.mode_id())?;
        if steps
            .iter()
            .any(|existing: &PlanStep| existing.step_id() == step.step_id())
        {
            return Err(diagnostic(
                ModeDiagnosticCode::DuplicateStep,
                Some(step.step_id()),
                "duplicate plan step id",
            ));
        }
        if steps.len() >= MAX_PLAN_STEPS {
            return Err(diagnostic(
                ModeDiagnosticCode::FieldTooLong,
                Some(descriptor.mode_id()),
                "too many plan steps",
            ));
        }
        steps.push(step);
    }
    if steps.is_empty() {
        return Err(diagnostic(
            ModeDiagnosticCode::MalformedMarker,
            Some(descriptor.mode_id()),
            "marked plan body contains no steps",
        ));
    }
    Ok(PlanExtraction {
        status: PlanExtractionStatus::Extracted,
        steps,
        diagnostic: None,
    })
}

/// Creates mode state from a validated extraction.
#[must_use]
pub fn mode_state_from_extraction(
    descriptor: &ModeDescriptor,
    extraction: &PlanExtraction,
) -> ModeState {
    let runtime_state = if extraction.steps().is_empty() {
        ModeRuntimeState::Inactive
    } else {
        ModeRuntimeState::Planning
    };
    ModeState {
        mode_id: Text::from(descriptor.mode_id()),
        runtime_state,
        steps: extraction.steps().to_vec(),
        active_loop: None,
        retry_class: ModeRetryClass::Unknown,
    }
}

/// Applies a selected-step update as pure state.
///
/// # Errors
///
/// Returns [`ModeDiagnostic`] when the step is unknown or the update epoch is
/// stale relative to the existing selected epoch.
pub fn apply_selected_step_update(
    state: &ModeState,
    draft: &SelectedStepUpdateDraft<'_>,
) -> ModeResult<ModeState> {
    let step_id = safe_text(STEP_ID_FIELD, draft.step_id, None)?;
    let mut next = state.clone();
    let Some(step) = next.steps.iter_mut().find(|item| item.step_id() == step_id) else {
        return Err(diagnostic(
            ModeDiagnosticCode::UnknownStep,
            Some(step_id.as_str()),
            "selected step is not in mode state",
        ));
    };
    if draft.update_epoch <= step.selected_epoch() {
        return Err(diagnostic(
            ModeDiagnosticCode::StaleUpdate,
            Some(step_id.as_str()),
            "selected-step update is stale",
        ));
    }
    step.status = draft.status;
    step.selected_epoch = draft.update_epoch;
    next.runtime_state = match draft.status {
        StepSelectionStatus::Pending => ModeRuntimeState::Planning,
        StepSelectionStatus::Selected => ModeRuntimeState::Selected,
        StepSelectionStatus::Completed => ModeRuntimeState::Completed,
    };
    Ok(next)
}

/// Starts loop continuation work through typed state only.
///
/// # Errors
///
/// Returns [`ModeDiagnostic`] for unsafe work metadata or invalid budgets.
pub fn start_loop_work(state: &ModeState, draft: &LoopConditionDraft<'_>) -> ModeResult<ModeState> {
    validate_loop_budget(draft.budget_units)?;
    let work_id = safe_text(WORK_ID_FIELD, draft.work_id, None)?;
    let reason = safe_bounded_text(
        LOOP_REASON_FIELD,
        draft.reason,
        MAX_STEP_LABEL_CHARS,
        Some(work_id.as_str()),
    )?;
    let mut next = state.clone();
    next.runtime_state = ModeRuntimeState::Looping;
    next.retry_class = draft.retry_class;
    next.active_loop = Some(LoopWorkItem {
        work_id,
        reason,
        budget_units: draft.budget_units,
        retry_class: draft.retry_class,
    });
    Ok(next)
}

/// Validates a breakout acknowledgement against active loop work.
#[must_use]
pub fn validate_breakout_acknowledgement(
    state: &ModeState,
    draft: &BreakoutAcknowledgementDraft<'_>,
) -> BreakoutAcknowledgement {
    let Ok(work_id) = safe_text(WORK_ID_FIELD, draft.work_id, None) else {
        return BreakoutAcknowledgement {
            status: BreakoutAckStatus::WorkMismatch,
            work_id: None,
            diagnostic: Some(diagnostic(
                ModeDiagnosticCode::UnsafeText,
                None,
                "unsafe breakout acknowledgement work id",
            )),
        };
    };
    let Some(active) = state.active_loop() else {
        return BreakoutAcknowledgement {
            status: BreakoutAckStatus::WorkMismatch,
            work_id: None,
            diagnostic: Some(diagnostic(
                ModeDiagnosticCode::UnknownStep,
                Some(work_id.as_str()),
                "no active loop work",
            )),
        };
    };
    if active.work_id() != work_id {
        return BreakoutAcknowledgement {
            status: BreakoutAckStatus::WorkMismatch,
            work_id: Some(work_id),
            diagnostic: Some(diagnostic(
                ModeDiagnosticCode::UnknownStep,
                Some(active.work_id()),
                "breakout acknowledgement does not match active loop work",
            )),
        };
    }
    if is_stale_ack(draft.acknowledgement_epoch, draft.current_epoch) {
        return BreakoutAcknowledgement {
            status: BreakoutAckStatus::Stale,
            work_id: Some(work_id),
            diagnostic: Some(diagnostic(
                ModeDiagnosticCode::StaleUpdate,
                Some(active.work_id()),
                "breakout acknowledgement is stale",
            )),
        };
    }
    let status = if draft.success {
        BreakoutAckStatus::Accepted
    } else {
        BreakoutAckStatus::WorkMismatch
    };
    BreakoutAcknowledgement {
        status,
        work_id: Some(work_id),
        diagnostic: None,
    }
}

/// Plans host-controller availability without installing commands.
#[must_use]
pub fn plan_mode_controller(
    descriptor: &ModeDescriptor,
    status: ModeControllerStatus,
) -> ModeControllerPlan {
    if status == ModeControllerStatus::Missing {
        return ModeControllerPlan {
            status: ModeControllerPlanStatus::MissingController,
            mode_id: Text::from(descriptor.mode_id()),
            installed_commands: Collection::new(),
            diagnostic: Some(diagnostic(
                ModeDiagnosticCode::MissingController,
                Some(descriptor.mode_id()),
                "host mode controller is missing",
            )),
        };
    }
    ModeControllerPlan {
        status: ModeControllerPlanStatus::Ready,
        mode_id: Text::from(descriptor.mode_id()),
        installed_commands: Collection::new(),
        diagnostic: None,
    }
}

/// Validates a durable safe mode receipt.
///
/// # Errors
///
/// Returns [`ModeDiagnostic`] when receipt metadata leaks unsafe content or the
/// state hash is not a lowercase BLAKE3 identity.
pub fn validate_mode_receipt(draft: &ModeReceiptDraft<'_>) -> ModeResult<ModeReceipt> {
    let mode_id = safe_text(MODE_ID_FIELD, draft.mode_id, None)?;
    let status = safe_text(RECEIPT_STATUS_FIELD, draft.status, Some(mode_id.as_str()))?;
    let state_hash = safe_hash(draft.state_hash)?;
    let issue_code = draft
        .issue_code
        .map(|issue| safe_text(RECEIPT_ISSUE_FIELD, issue, Some(mode_id.as_str())))
        .transpose()?;
    Ok(ModeReceipt {
        mode_id,
        status,
        state_hash,
        issue_code,
    })
}

fn parse_step_line(line: &str, mode_id: &str) -> ModeResult<PlanStep> {
    if !line.starts_with(PLAN_STEP_PREFIX) {
        return Err(diagnostic(
            ModeDiagnosticCode::MalformedMarker,
            Some(mode_id),
            "plan step must use explicit step marker",
        ));
    }
    let id_start = PLAN_STEP_PREFIX.len();
    let Some(id_end_offset) = line[id_start..].find(PLAN_STEP_SEPARATOR) else {
        return Err(diagnostic(
            ModeDiagnosticCode::MalformedMarker,
            Some(mode_id),
            "plan step marker is missing closing bracket",
        ));
    };
    let id_end = id_start + id_end_offset;
    let step_id = safe_text(STEP_ID_FIELD, &line[id_start..id_end], Some(mode_id))?;
    let label = safe_bounded_text(
        STEP_LABEL_FIELD,
        &line[id_end + PLAN_STEP_SEPARATOR.len()..],
        MAX_STEP_LABEL_CHARS,
        Some(step_id.as_str()),
    )?;
    Ok(PlanStep {
        step_id,
        label,
        status: StepSelectionStatus::Pending,
        selected_epoch: INITIAL_STEP_EPOCH,
    })
}

fn validate_plan_text_size(text: &str) -> ModeResult<()> {
    let actual_chars = text.chars().count();
    if actual_chars > MAX_PLAN_BODY_CHARS {
        return Err(diagnostic(
            ModeDiagnosticCode::FieldTooLong,
            None,
            "plan body is too large",
        ));
    }
    Ok(())
}

fn validate_loop_budget(budget: u16) -> ModeResult<()> {
    if !(MIN_LOOP_BUDGET_UNITS..=MAX_LOOP_BUDGET_UNITS).contains(&budget) {
        return Err(diagnostic(
            ModeDiagnosticCode::InvalidBudget,
            None,
            "loop budget is outside bounds",
        ));
    }
    Ok(())
}

fn safe_output_contract(value: &str) -> ModeResult<Text> {
    let output_contract = safe_text(OUTPUT_CONTRACT_FIELD, value, None)?;
    match output_contract.as_str() {
        SELECTED_STEP_CONTRACT | LOOP_WORK_CONTRACT => Ok(output_contract),
        _ => Err(diagnostic(
            ModeDiagnosticCode::UnsupportedOutputContract,
            None,
            "unsupported mode output contract",
        )),
    }
}

fn safe_bounded_text(
    field: FieldName,
    value: &str,
    max_chars: usize,
    subject_id: Option<&str>,
) -> ModeResult<Text> {
    let text = safe_text(field, value, subject_id)?;
    if text.chars().count() > max_chars {
        return Err(diagnostic(
            ModeDiagnosticCode::FieldTooLong,
            subject_id,
            field.as_str(),
        ));
    }
    Ok(text)
}

fn safe_text(field: FieldName, value: &str, subject_id: Option<&str>) -> ModeResult<Text> {
    let normalized = validation::normalized_adapter_field(field, value)
        .map_err(|error| map_adapter_error(&error))?;
    let lower = normalized.to_ascii_lowercase();
    if UNSAFE_MODE_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Err(diagnostic(
            ModeDiagnosticCode::UnsafeText,
            subject_id,
            "mode metadata contains unsafe marker",
        ));
    }
    Ok(normalized)
}

fn safe_hash(value: &str) -> ModeResult<Text> {
    let normalized = validation::normalized_adapter_field(HASH_FIELD, value)
        .map_err(|error| map_adapter_error(&error))?;
    if Hash::new(&normalized).is_ok() {
        Ok(normalized)
    } else {
        Err(diagnostic(
            ModeDiagnosticCode::InvalidHash,
            None,
            HASH_FIELD.as_str(),
        ))
    }
}

fn is_stale_ack(acknowledgement_epoch: u32, current_epoch: u32) -> bool {
    current_epoch.saturating_sub(acknowledgement_epoch) > STALE_ACK_EPOCH_DELTA
}

fn map_adapter_error(error: &AdapterContractError) -> ModeDiagnostic {
    match error {
        AdapterContractError::EmptyField { field } => {
            diagnostic(ModeDiagnosticCode::EmptyField, None, field)
        }
        AdapterContractError::FieldTooLong { field, .. } => {
            diagnostic(ModeDiagnosticCode::FieldTooLong, None, field)
        }
        AdapterContractError::NulCharacter { field, .. } => {
            diagnostic(ModeDiagnosticCode::NulCharacter, None, field)
        }
        AdapterContractError::InvalidEvidenceHash { field, .. } => {
            diagnostic(ModeDiagnosticCode::InvalidHash, None, field)
        }
        AdapterContractError::EmptyCollection { field } => {
            diagnostic(ModeDiagnosticCode::MalformedMarker, None, field)
        }
        AdapterContractError::UnsupportedStatusState { field, .. }
        | AdapterContractError::UnsupportedStatusObservationKind { field, .. }
        | AdapterContractError::UnsupportedSessionReferenceKind { field, .. }
        | AdapterContractError::UnsupportedWorkspaceValue { field, .. }
        | AdapterContractError::UnsupportedDataspaceValue { field, .. }
        | AdapterContractError::UnsupportedRouterValue { field, .. } => {
            diagnostic(ModeDiagnosticCode::UnsafeText, None, field)
        }
    }
}

fn diagnostic(code: ModeDiagnosticCode, subject_id: Option<&str>, message: &str) -> ModeDiagnostic {
    ModeDiagnostic {
        code,
        subject_id: subject_id.map(Text::from),
        message: Text::from(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODE_ID: &str = "host-plan";
    const MODE_LABEL: &str = "Host planning";
    const CAPABILITY_CEILING: &str = "read-only-guidance";
    const PLAN_START: &str = "<!-- animus-plan:start -->";
    const PLAN_END: &str = "<!-- animus-plan:end -->";
    const STEP_ONE: &str = "inspect";
    const STEP_TWO: &str = "validate";
    const PLAN_TEXT: &str = "intro\n<!-- animus-plan:start -->\n- [inspect] Inspect bounded context\n- [validate] Run focused checks\n<!-- animus-plan:end -->";
    const NUMBERED_PROSE: &str = "1. This is explanation.\n2. It is not inside markers.";
    const LOOP_WORK_ID: &str = "loop-validate";
    const LOOP_REASON: &str = "validation follow-up";
    const CURRENT_EPOCH: u32 = 7;
    const STALE_EPOCH: u32 = 1;
    const FRESH_EPOCH: u32 = 7;
    const SELECTED_EPOCH: u32 = 2;
    const LOOP_BUDGET: u16 = 3;
    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn descriptor_draft() -> ModeDescriptorDraft<'static> {
        ModeDescriptorDraft {
            mode_id: MODE_ID,
            label: MODE_LABEL,
            posture: ModePosture::PlanningReadOnly,
            capability_ceiling: CAPABILITY_CEILING,
            plan_start_marker: PLAN_START,
            plan_end_marker: PLAN_END,
            output_contract: SELECTED_STEP_CONTRACT,
            loop_budget_units: LOOP_BUDGET,
            trusted: true,
        }
    }

    fn descriptor() -> ModeDescriptor {
        validate_mode_descriptor(&descriptor_draft()).unwrap()
    }

    fn state() -> ModeState {
        let descriptor = descriptor();
        let extraction = extract_plan_steps(&descriptor, PLAN_TEXT).unwrap();
        mode_state_from_extraction(&descriptor, &extraction)
    }

    #[test]
    fn host_defined_mode_descriptor_validates_without_commands() {
        // r[verify agent_harness_runtime.hermes_plan_loop_modes.fsm]
        // r[verify agent_harness_runtime.hermes_plan_loop_modes.host_owned]
        let descriptor = descriptor();
        let missing_controller = plan_mode_controller(&descriptor, ModeControllerStatus::Missing);

        assert_eq!(descriptor.mode_id(), MODE_ID);
        assert_eq!(descriptor.posture(), ModePosture::PlanningReadOnly);
        assert_eq!(descriptor.output_contract(), SELECTED_STEP_CONTRACT);
        assert_eq!(
            missing_controller.status(),
            ModeControllerPlanStatus::MissingController
        );
        assert!(missing_controller.installed_commands().is_empty());
        assert!(PROHIBITED_PLAN_LOOP_EFFECTS.contains(&"command registration"));
    }

    #[test]
    fn plan_extraction_requires_explicit_markers_and_step_records() {
        // r[verify agent_harness_runtime.hermes_plan_loop_modes.plan_extraction]
        let descriptor = descriptor();
        let extraction = extract_plan_steps(&descriptor, PLAN_TEXT).unwrap();
        let no_plan = extract_plan_steps(&descriptor, NUMBERED_PROSE).unwrap();
        let malformed = extract_plan_steps(
            &descriptor,
            "<!-- animus-plan:start -->\n1. not a step\n<!-- animus-plan:end -->",
        )
        .expect_err("numbered list inside markers must fail explicit record parsing");

        assert_eq!(extraction.status(), PlanExtractionStatus::Extracted);
        assert_eq!(extraction.steps().len(), 2);
        assert_eq!(no_plan.status(), PlanExtractionStatus::NoPlan);
        assert_eq!(malformed.code(), ModeDiagnosticCode::MalformedMarker);
    }

    #[test]
    fn duplicate_and_unsafe_plan_steps_fail_extraction() {
        // r[verify agent_harness_runtime.hermes_plan_loop_modes.verification]
        let descriptor = descriptor();
        let duplicate = extract_plan_steps(
            &descriptor,
            "<!-- animus-plan:start -->\n- [inspect] First\n- [inspect] Second\n<!-- animus-plan:end -->",
        )
        .expect_err("duplicate steps must fail");
        let unsafe_text = extract_plan_steps(
            &descriptor,
            "<!-- animus-plan:start -->\n- [inspect] provider probe secret\n<!-- animus-plan:end -->",
        )
        .expect_err("unsafe step text must fail");

        assert_eq!(duplicate.code(), ModeDiagnosticCode::DuplicateStep);
        assert_eq!(unsafe_text.code(), ModeDiagnosticCode::UnsafeText);
    }

    #[test]
    fn selected_step_updates_reject_unknown_or_stale_steps() {
        // r[verify agent_harness_runtime.hermes_plan_loop_modes.plan_extraction]
        let state = state();
        let selected = apply_selected_step_update(
            &state,
            &SelectedStepUpdateDraft {
                step_id: STEP_ONE,
                status: StepSelectionStatus::Selected,
                update_epoch: SELECTED_EPOCH,
            },
        )
        .unwrap();
        let stale_update = apply_selected_step_update(
            &selected,
            &SelectedStepUpdateDraft {
                step_id: STEP_ONE,
                status: StepSelectionStatus::Completed,
                update_epoch: SELECTED_EPOCH,
            },
        )
        .expect_err("stale selected-step update must fail");
        let unknown = apply_selected_step_update(
            &state,
            &SelectedStepUpdateDraft {
                step_id: "missing-step",
                status: StepSelectionStatus::Selected,
                update_epoch: SELECTED_EPOCH,
            },
        )
        .expect_err("unknown selected step must fail");

        assert_eq!(selected.runtime_state(), ModeRuntimeState::Selected);
        assert_eq!(selected.steps()[0].status(), StepSelectionStatus::Selected);
        assert_eq!(stale_update.code(), ModeDiagnosticCode::StaleUpdate);
        assert_eq!(unknown.code(), ModeDiagnosticCode::UnknownStep);
    }

    #[test]
    fn loop_continuation_and_breakout_acknowledgement_match_active_work() {
        // r[verify agent_harness_runtime.hermes_plan_loop_modes.continuation]
        let loop_state = start_loop_work(
            &state(),
            &LoopConditionDraft {
                work_id: LOOP_WORK_ID,
                reason: LOOP_REASON,
                budget_units: LOOP_BUDGET,
                retry_class: ModeRetryClass::Transient,
            },
        )
        .unwrap();
        let accepted = validate_breakout_acknowledgement(
            &loop_state,
            &BreakoutAcknowledgementDraft {
                work_id: LOOP_WORK_ID,
                success: true,
                acknowledgement_epoch: FRESH_EPOCH,
                current_epoch: CURRENT_EPOCH,
            },
        );
        let mismatch = validate_breakout_acknowledgement(
            &loop_state,
            &BreakoutAcknowledgementDraft {
                work_id: STEP_TWO,
                success: true,
                acknowledgement_epoch: FRESH_EPOCH,
                current_epoch: CURRENT_EPOCH,
            },
        );
        let stale = validate_breakout_acknowledgement(
            &loop_state,
            &BreakoutAcknowledgementDraft {
                work_id: LOOP_WORK_ID,
                success: true,
                acknowledgement_epoch: STALE_EPOCH,
                current_epoch: CURRENT_EPOCH,
            },
        );

        assert_eq!(loop_state.runtime_state(), ModeRuntimeState::Looping);
        assert_eq!(loop_state.retry_class(), ModeRetryClass::Transient);
        assert_eq!(accepted.status(), BreakoutAckStatus::Accepted);
        assert_eq!(mismatch.status(), BreakoutAckStatus::WorkMismatch);
        assert_eq!(stale.status(), BreakoutAckStatus::Stale);
    }

    #[test]
    fn receipts_are_hash_backed_and_reject_leaky_issue_codes() {
        // r[verify agent_harness_runtime.hermes_plan_loop_modes.verification]
        let receipt = validate_mode_receipt(&ModeReceiptDraft {
            mode_id: MODE_ID,
            status: "completed",
            state_hash: HASH_A,
            issue_code: Some("mode-complete"),
        })
        .unwrap();
        let leaking = validate_mode_receipt(&ModeReceiptDraft {
            mode_id: MODE_ID,
            status: "failed",
            state_hash: HASH_A,
            issue_code: Some("raw prompt leaked"),
        })
        .expect_err("receipt issue code must be safe");

        assert_eq!(receipt.mode_id(), MODE_ID);
        assert_eq!(receipt.state_hash(), HASH_A);
        assert_eq!(leaking.code(), ModeDiagnosticCode::UnsafeText);
    }
}
