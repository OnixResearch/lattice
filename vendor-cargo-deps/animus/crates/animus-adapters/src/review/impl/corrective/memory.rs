const CORRECTIVE_MIN_PROMPT_BUDGET_UNITS: u16 = 1;
const CORRECTIVE_DEFAULT_PROMOTION_THRESHOLD: usize = 2;
const CORRECTIVE_HASH_DOMAIN: &str = "animus.corrective-memory.v1";
const CORRECTIVE_ACTION_HASH_DOMAIN: &str = "animus.corrective-memory.action.v1";
const PROMOTION_TARGET_DETERMINISTIC_CHECK: &str = "deterministic-check";
const PROMOTION_TARGET_CHECK_ALIAS: &str = "check";
const PROMOTION_TARGET_POLICY_GUIDANCE: &str = "policy-guidance";
const PROMOTION_TARGET_SPEC_RULE: &str = "spec-rule";
const PROMOTION_TARGET_MEMORY_GUIDANCE: &str = "memory-guidance";
const PROMOTION_TARGET_HUMAN_CHECKPOINT: &str = "human-checkpoint";
const RECEIPT_STATUS_ACCEPTED: &str = "accepted";
const RECEIPT_STATUS_DENIED: &str = "denied";
const RECEIPT_STATUS_MISSING_STORE: &str = "missing-store";
const RECEIPT_STATUS_UNAVAILABLE_STORE: &str = "unavailable-store";

/// Corrective-memory record kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrectiveRecordKind {
    /// A mistake to avoid repeating.
    Mistake,
    /// A user or operator preference.
    Preference,
    /// A working pattern that should be repeated.
    WorkingPattern,
    /// A failing pattern that should be avoided.
    FailingPattern,
    /// A domain note supplied for future context.
    DomainNote,
}

impl CorrectiveRecordKind {
    /// Returns the stable kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mistake => "mistake",
            Self::Preference => "preference",
            Self::WorkingPattern => "working-pattern",
            Self::FailingPattern => "failing-pattern",
            Self::DomainNote => "domain-note",
        }
    }
}

/// Trust scope attached to a corrective record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrectiveTrustScope {
    /// User-level corrective record.
    User,
    /// Project-level corrective record.
    Project,
    /// Session-level corrective record.
    Session,
    /// Host/application-level corrective record.
    Host,
}

impl CorrectiveTrustScope {
    /// Returns the stable scope label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Project => "project",
            Self::Session => "session",
            Self::Host => "host",
        }
    }
}

/// Freshness classification for a corrective record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrectiveFreshness {
    /// Fresh enough for prompt influence.
    Fresh,
    /// Stale unless selected policy allows stale records.
    Stale,
    /// Expired and not prompt-admissible.
    Expired,
}

impl CorrectiveFreshness {
    /// Returns the stable freshness label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Stale => "stale",
            Self::Expired => "expired",
        }
    }
}

/// Prompt influence policy requested by a corrective record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrectiveInclusionPolicy {
    /// Host must first select the record as relevant.
    WhenRelevant,
    /// Never include the record in model-visible prompt context.
    Never,
    /// Rejected compatibility shape for always-read memories.
    AlwaysRead,
}

impl CorrectiveInclusionPolicy {
    /// Returns the stable inclusion label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WhenRelevant => "when-relevant",
            Self::Never => "never",
            Self::AlwaysRead => "always-read",
        }
    }
}

/// Redaction state for corrective-record body material.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrectiveRedactionState {
    /// Body identity may influence prompt assembly when admitted.
    Public,
    /// Only metadata may be exposed.
    MetadataOnly,
    /// Secret material that cannot influence model-visible prompts.
    Secret,
}

impl CorrectiveRedactionState {
    /// Returns the stable redaction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::MetadataOnly => "metadata-only",
            Self::Secret => "secret",
        }
    }
}

/// Borrowed corrective-record draft supplied by a host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CorrectiveRecordDraft<'a> {
    /// Safe host-local record identity.
    pub record_id: &'a str,
    /// Corrective record kind.
    pub kind: CorrectiveRecordKind,
    /// Safe source label for the host-owned store or packet.
    pub source: &'a str,
    /// Trust scope declared by the host.
    pub trust_scope: CorrectiveTrustScope,
    /// Safe subject label.
    pub subject: &'a str,
    /// Safe finding or guidance class label.
    pub class: &'a str,
    /// Safe scope label.
    pub scope: &'a str,
    /// Safe route label.
    pub route: &'a str,
    /// Freshness class.
    pub freshness: CorrectiveFreshness,
    /// Prompt-inclusion policy.
    pub inclusion: CorrectiveInclusionPolicy,
    /// Prompt budget units consumed if admitted.
    pub prompt_budget_units: u16,
    /// BLAKE3 body identity; never the raw body.
    pub body_hash: &'a str,
    /// Redaction state for body material.
    pub redaction: CorrectiveRedactionState,
}

/// Validated corrective-memory descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorrectiveRecord {
    record_id: String,
    kind: CorrectiveRecordKind,
    source: String,
    trust_scope: CorrectiveTrustScope,
    subject: String,
    class: String,
    scope: String,
    route: String,
    freshness: CorrectiveFreshness,
    inclusion: CorrectiveInclusionPolicy,
    prompt_budget_units: u16,
    body_hash: String,
    redaction: CorrectiveRedactionState,
}

impl CorrectiveRecord {
    /// Returns the safe record identity.
    #[must_use]
    pub fn record_id(&self) -> &str {
        self.record_id.as_str()
    }

    /// Returns the record kind.
    #[must_use]
    pub const fn kind(&self) -> CorrectiveRecordKind {
        self.kind
    }

    /// Returns the safe source label.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// Returns the trust scope.
    #[must_use]
    pub const fn trust_scope(&self) -> CorrectiveTrustScope {
        self.trust_scope
    }

    /// Returns the safe subject label.
    #[must_use]
    pub fn subject(&self) -> &str {
        self.subject.as_str()
    }

    /// Returns the safe class label.
    #[must_use]
    pub fn class(&self) -> &str {
        self.class.as_str()
    }

    /// Returns the safe scope label.
    #[must_use]
    pub fn scope(&self) -> &str {
        self.scope.as_str()
    }

    /// Returns the safe route label.
    #[must_use]
    pub fn route(&self) -> &str {
        self.route.as_str()
    }

    /// Returns the freshness class.
    #[must_use]
    pub const fn freshness(&self) -> CorrectiveFreshness {
        self.freshness
    }

    /// Returns the inclusion policy.
    #[must_use]
    pub const fn inclusion(&self) -> CorrectiveInclusionPolicy {
        self.inclusion
    }

    /// Returns prompt budget units consumed if admitted.
    #[must_use]
    pub const fn prompt_budget_units(&self) -> u16 {
        self.prompt_budget_units
    }

    /// Returns the BLAKE3 body hash.
    #[must_use]
    pub fn body_hash(&self) -> &str {
        self.body_hash.as_str()
    }

    /// Returns the redaction state.
    #[must_use]
    pub const fn redaction(&self) -> CorrectiveRedactionState {
        self.redaction
    }

    /// Returns a safe BLAKE3 identity for receipts.
    #[must_use]
    pub fn record_hash(&self) -> String {
        corrective_record_hash(self)
    }
}

/// Policy for admitting corrective records into prompt influence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CorrectiveAdmissionPolicy<'a> {
    /// Source labels explicitly trusted by the host.
    pub trusted_sources: &'a [&'a str],
    /// Trust scopes allowed by the host.
    pub allowed_scopes: &'a [CorrectiveTrustScope],
    /// Whether stale records may influence prompts.
    pub allow_stale: bool,
    /// Remaining prompt budget units for corrective records.
    pub prompt_budget_units: u16,
}

/// Prompt-admission status for a corrective record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrectiveAdmissionStatus {
    /// Record may influence prompt assembly through a host-selected summary.
    Admitted,
    /// Record is safely omitted without prompt influence.
    Omitted,
    /// Record is denied because policy would be unsafe to satisfy.
    Denied,
}

/// Prompt-admission reason for a corrective record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrectiveAdmissionReason {
    /// Record satisfied all admission inputs.
    Accepted,
    /// Always-read corrective records are rejected.
    AlwaysReadRejected,
    /// Source label was not explicitly trusted.
    SourceNotTrusted,
    /// Trust scope was not allowed.
    ScopeDenied,
    /// Record was stale and stale records were disabled.
    StaleRecord,
    /// Record was expired.
    ExpiredRecord,
    /// Prompt budget was exhausted or insufficient.
    BudgetExceeded,
    /// Inclusion policy disabled prompt influence.
    InclusionDisabled,
    /// Redaction state allows metadata only.
    MetadataOnly,
    /// Redaction state forbids prompt influence.
    SecretRedaction,
}

impl CorrectiveAdmissionReason {
    /// Returns the stable reason label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::AlwaysReadRejected => "always-read-rejected",
            Self::SourceNotTrusted => "source-not-trusted",
            Self::ScopeDenied => "scope-denied",
            Self::StaleRecord => "stale-record",
            Self::ExpiredRecord => "expired-record",
            Self::BudgetExceeded => "budget-exceeded",
            Self::InclusionDisabled => "inclusion-disabled",
            Self::MetadataOnly => "metadata-only",
            Self::SecretRedaction => "secret-redaction",
        }
    }
}

/// Corrective-record prompt-admission decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorrectiveAdmissionDecision {
    record_hash: String,
    status: CorrectiveAdmissionStatus,
    reason: CorrectiveAdmissionReason,
    consumed_budget_units: u16,
}

impl CorrectiveAdmissionDecision {
    /// Returns the safe record identity hash.
    #[must_use]
    pub fn record_hash(&self) -> &str {
        self.record_hash.as_str()
    }

    /// Returns admission status.
    #[must_use]
    pub const fn status(&self) -> CorrectiveAdmissionStatus {
        self.status
    }

    /// Returns admission reason.
    #[must_use]
    pub const fn reason(&self) -> CorrectiveAdmissionReason {
        self.reason
    }

    /// Returns prompt budget units consumed by this decision.
    #[must_use]
    pub const fn consumed_budget_units(&self) -> u16 {
        self.consumed_budget_units
    }
}

/// Suggested target for a corrective promotion candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrectivePromotionTarget {
    /// Add or extend a deterministic check.
    DeterministicCheck,
    /// Add or update policy guidance.
    PolicyGuidance,
    /// Add or update a lifecycle/spec rule.
    SpecRule,
    /// Add bounded memory guidance.
    MemoryGuidance,
    /// Route to a human checkpoint.
    HumanCheckpoint,
}

impl CorrectivePromotionTarget {
    /// Returns the stable target label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeterministicCheck => PROMOTION_TARGET_DETERMINISTIC_CHECK,
            Self::PolicyGuidance => PROMOTION_TARGET_POLICY_GUIDANCE,
            Self::SpecRule => PROMOTION_TARGET_SPEC_RULE,
            Self::MemoryGuidance => PROMOTION_TARGET_MEMORY_GUIDANCE,
            Self::HumanCheckpoint => PROMOTION_TARGET_HUMAN_CHECKPOINT,
        }
    }
}

/// Corrective promotion candidate routed from repeated review metrics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorrectivePromotionCandidate {
    class: String,
    scope: String,
    route: String,
    target: CorrectivePromotionTarget,
    count: usize,
    evidence_hashes: Vec<String>,
}

impl CorrectivePromotionCandidate {
    /// Returns the repeated class label.
    #[must_use]
    pub fn class(&self) -> &str {
        self.class.as_str()
    }

    /// Returns the repeated scope label.
    #[must_use]
    pub fn scope(&self) -> &str {
        self.scope.as_str()
    }

    /// Returns the route label.
    #[must_use]
    pub fn route(&self) -> &str {
        self.route.as_str()
    }

    /// Returns the suggested promotion target.
    #[must_use]
    pub const fn target(&self) -> CorrectivePromotionTarget {
        self.target
    }

    /// Returns repeated finding count.
    #[must_use]
    pub const fn count(&self) -> usize {
        self.count
    }

    /// Returns representative safe evidence hashes.
    #[must_use]
    pub fn evidence_hashes(&self) -> &[String] {
        self.evidence_hashes.as_slice()
    }

    /// Returns a safe BLAKE3 identity for action requests.
    #[must_use]
    pub fn candidate_hash(&self) -> String {
        corrective_candidate_hash(self)
    }
}

/// Availability of a host-owned corrective store.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrectiveStoreStatus {
    /// Store is available for an explicit host action.
    Available,
    /// No store adapter was injected.
    Missing,
    /// Store adapter exists but is unavailable.
    Unavailable,
}

/// Corrective-memory host action kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrectiveHostActionKind {
    /// Append a corrective record.
    AppendRecord,
    /// Promote a repeated-finding candidate.
    PromoteCandidate,
}

impl CorrectiveHostActionKind {
    /// Returns the stable action-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AppendRecord => "append-record",
            Self::PromoteCandidate => "promote-candidate",
        }
    }
}

/// Borrowed append request for a host-owned corrective store.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CorrectiveAppendRequestDraft<'a> {
    /// Safe action identity.
    pub action_id: &'a str,
    /// Safe host store label.
    pub store_label: &'a str,
    /// Safe actor or policy label.
    pub actor_label: &'a str,
    /// Store availability status.
    pub store_status: CorrectiveStoreStatus,
    /// Record being requested for append.
    pub record: &'a CorrectiveRecord,
}

/// Validated append request for a host-owned corrective store.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorrectiveAppendRequest {
    action_id: String,
    store_label: String,
    actor_label: String,
    record_hash: String,
}

impl CorrectiveAppendRequest {
    /// Returns the safe action identity.
    #[must_use]
    pub fn action_id(&self) -> &str {
        self.action_id.as_str()
    }

    /// Returns the safe store label.
    #[must_use]
    pub fn store_label(&self) -> &str {
        self.store_label.as_str()
    }

    /// Returns the safe actor label.
    #[must_use]
    pub fn actor_label(&self) -> &str {
        self.actor_label.as_str()
    }

    /// Returns the safe record hash.
    #[must_use]
    pub fn record_hash(&self) -> &str {
        self.record_hash.as_str()
    }
}

/// Borrowed promotion request for a host-owned corrective store.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CorrectivePromoteRequestDraft<'a> {
    /// Safe action identity.
    pub action_id: &'a str,
    /// Safe host store label.
    pub store_label: &'a str,
    /// Safe actor or policy label.
    pub actor_label: &'a str,
    /// Store availability status.
    pub store_status: CorrectiveStoreStatus,
    /// Promotion candidate under review.
    pub candidate: &'a CorrectivePromotionCandidate,
}

/// Validated promotion request for a host-owned corrective store.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorrectivePromoteRequest {
    action_id: String,
    store_label: String,
    actor_label: String,
    candidate_hash: String,
    target: CorrectivePromotionTarget,
}

impl CorrectivePromoteRequest {
    /// Returns the safe action identity.
    #[must_use]
    pub fn action_id(&self) -> &str {
        self.action_id.as_str()
    }

    /// Returns the safe store label.
    #[must_use]
    pub fn store_label(&self) -> &str {
        self.store_label.as_str()
    }

    /// Returns the safe actor label.
    #[must_use]
    pub fn actor_label(&self) -> &str {
        self.actor_label.as_str()
    }

    /// Returns the safe candidate hash.
    #[must_use]
    pub fn candidate_hash(&self) -> &str {
        self.candidate_hash.as_str()
    }

    /// Returns the promotion target.
    #[must_use]
    pub const fn target(&self) -> CorrectivePromotionTarget {
        self.target
    }
}

/// Corrective host-action receipt status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorrectiveHostActionReceiptStatus {
    /// Host accepted the explicit action request.
    Accepted,
    /// Host denied the explicit action request.
    Denied,
    /// No host store adapter was injected.
    MissingStore,
    /// Host store was unavailable.
    UnavailableStore,
}

impl CorrectiveHostActionReceiptStatus {
    /// Returns the stable receipt status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => RECEIPT_STATUS_ACCEPTED,
            Self::Denied => RECEIPT_STATUS_DENIED,
            Self::MissingStore => RECEIPT_STATUS_MISSING_STORE,
            Self::UnavailableStore => RECEIPT_STATUS_UNAVAILABLE_STORE,
        }
    }
}

/// Borrowed corrective host-action receipt draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CorrectiveHostActionReceiptDraft<'a> {
    /// Action kind.
    pub action_kind: CorrectiveHostActionKind,
    /// Safe store label.
    pub store_label: &'a str,
    /// Receipt status.
    pub status: CorrectiveHostActionReceiptStatus,
    /// Optional safe issue code.
    pub issue_code: Option<&'a str>,
    /// Safe BLAKE3 evidence hashes.
    pub evidence_hashes: &'a [&'a str],
}

/// Safe corrective host-action receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorrectiveHostActionReceipt {
    receipt_hash: String,
    action_kind: CorrectiveHostActionKind,
    store_label: String,
    status: CorrectiveHostActionReceiptStatus,
    issue_code: Option<String>,
    evidence_hashes: Vec<String>,
}

impl CorrectiveHostActionReceipt {
    /// Returns the deterministic BLAKE3 receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }

    /// Returns the action kind.
    #[must_use]
    pub const fn action_kind(&self) -> CorrectiveHostActionKind {
        self.action_kind
    }

    /// Returns the safe store label.
    #[must_use]
    pub fn store_label(&self) -> &str {
        self.store_label.as_str()
    }

    /// Returns the receipt status.
    #[must_use]
    pub const fn status(&self) -> CorrectiveHostActionReceiptStatus {
        self.status
    }

    /// Returns the optional safe issue code.
    #[must_use]
    pub fn issue_code(&self) -> Option<&str> {
        self.issue_code.as_deref()
    }

    /// Returns safe evidence hashes.
    #[must_use]
    pub fn evidence_hashes(&self) -> &[String] {
        self.evidence_hashes.as_slice()
    }
}

/// Validates a corrective-memory record.
///
/// # Errors
///
/// Returns [`ReviewReportError`] when safe labels, prompt budget, or body hash
/// validation fails.
pub fn validate_corrective_record(
    draft: &CorrectiveRecordDraft<'_>,
) -> Result<CorrectiveRecord, ReviewReportError> {
    let record_id = validate_safe_metadata("record_id", draft.record_id)?;
    let source = validate_safe_metadata("source", draft.source)?;
    let subject = validate_safe_metadata("subject", draft.subject)?;
    let class = validate_safe_metadata(TAG_CLASS, draft.class)?;
    let scope = validate_safe_metadata(TAG_SCOPE, draft.scope)?;
    let route = validate_safe_metadata(TAG_ROUTE, draft.route)?;
    if draft.prompt_budget_units < CORRECTIVE_MIN_PROMPT_BUDGET_UNITS {
        return Err(ReviewReportError::new(
            ReviewDiagnosticCode::PromptAdmissionDenied,
            "prompt_budget_units",
        ));
    }
    let body_hash = validate_blake3_hash("body_hash", draft.body_hash)?;
    Ok(CorrectiveRecord {
        record_id,
        kind: draft.kind,
        source,
        trust_scope: draft.trust_scope,
        subject,
        class,
        scope,
        route,
        freshness: draft.freshness,
        inclusion: draft.inclusion,
        prompt_budget_units: draft.prompt_budget_units,
        body_hash,
        redaction: draft.redaction,
    })
}

/// Evaluates whether a corrective record may influence prompt assembly.
#[must_use]
pub fn admit_corrective_record(
    record: &CorrectiveRecord,
    policy: &CorrectiveAdmissionPolicy<'_>,
) -> CorrectiveAdmissionDecision {
    let reason = corrective_denial_reason(record, policy);
    let (status, consumed_budget_units) = match reason {
        CorrectiveAdmissionReason::Accepted => {
            (CorrectiveAdmissionStatus::Admitted, record.prompt_budget_units())
        }
        CorrectiveAdmissionReason::BudgetExceeded
        | CorrectiveAdmissionReason::InclusionDisabled
        | CorrectiveAdmissionReason::MetadataOnly
        | CorrectiveAdmissionReason::StaleRecord => (CorrectiveAdmissionStatus::Omitted, 0),
        CorrectiveAdmissionReason::AlwaysReadRejected
        | CorrectiveAdmissionReason::SourceNotTrusted
        | CorrectiveAdmissionReason::ScopeDenied
        | CorrectiveAdmissionReason::ExpiredRecord
        | CorrectiveAdmissionReason::SecretRedaction => (CorrectiveAdmissionStatus::Denied, 0),
    };
    CorrectiveAdmissionDecision {
        record_hash: record.record_hash(),
        status,
        reason,
        consumed_budget_units,
    }
}

/// Routes repeated review-metric findings into corrective promotion candidates.
///
/// # Errors
///
/// Returns [`ReviewReportError`] when a repeated finding uses an unsupported
/// promotion route label.
pub fn corrective_promotion_candidates(
    records: &[ReviewMetricRecord],
    minimum_count: usize,
) -> Result<Vec<CorrectivePromotionCandidate>, ReviewReportError> {
    let threshold = minimum_count.max(CORRECTIVE_DEFAULT_PROMOTION_THRESHOLD);
    let review_candidates = review_promotion_candidates(records, threshold);
    let mut candidates = Vec::with_capacity(review_candidates.len());
    for candidate in review_candidates {
        let target = promotion_target_from_route(candidate.route())?;
        candidates.push(CorrectivePromotionCandidate {
            class: candidate.class().to_owned(),
            scope: candidate.scope().to_owned(),
            route: candidate.route().to_owned(),
            target,
            count: candidate.count(),
            evidence_hashes: candidate.example_hashes().to_vec(),
        });
    }
    Ok(candidates)
}

/// Validates an explicit corrective-record append request.
///
/// # Errors
///
/// Returns [`ReviewReportError`] when the host store is missing/unavailable or
/// request metadata is unsafe.
pub fn validate_corrective_append_request(
    draft: &CorrectiveAppendRequestDraft<'_>,
) -> Result<CorrectiveAppendRequest, ReviewReportError> {
    validate_store_available(draft.store_status)?;
    Ok(CorrectiveAppendRequest {
        action_id: validate_safe_metadata("action_id", draft.action_id)?,
        store_label: validate_safe_metadata("store_label", draft.store_label)?,
        actor_label: validate_safe_metadata("actor_label", draft.actor_label)?,
        record_hash: draft.record.record_hash(),
    })
}

/// Validates an explicit corrective-promotion request.
///
/// # Errors
///
/// Returns [`ReviewReportError`] when the host store is missing/unavailable or
/// request metadata is unsafe.
pub fn validate_corrective_promote_request(
    draft: &CorrectivePromoteRequestDraft<'_>,
) -> Result<CorrectivePromoteRequest, ReviewReportError> {
    validate_store_available(draft.store_status)?;
    Ok(CorrectivePromoteRequest {
        action_id: validate_safe_metadata("action_id", draft.action_id)?,
        store_label: validate_safe_metadata("store_label", draft.store_label)?,
        actor_label: validate_safe_metadata("actor_label", draft.actor_label)?,
        candidate_hash: draft.candidate.candidate_hash(),
        target: draft.candidate.target(),
    })
}

/// Validates a safe corrective host-action receipt.
///
/// # Errors
///
/// Returns [`ReviewReportError`] when metadata leaks unsafe content or an
/// evidence hash is not BLAKE3-shaped.
pub fn validate_corrective_host_action_receipt(
    draft: &CorrectiveHostActionReceiptDraft<'_>,
) -> Result<CorrectiveHostActionReceipt, ReviewReportError> {
    let store_label = validate_safe_metadata("store_label", draft.store_label)?;
    let issue_code = draft
        .issue_code
        .map(|issue| validate_safe_metadata("issue_code", issue))
        .transpose()?;
    let mut evidence_hashes = Vec::with_capacity(draft.evidence_hashes.len());
    for evidence_hash in draft.evidence_hashes {
        evidence_hashes.push(validate_blake3_hash("evidence_hash", evidence_hash)?);
    }
    let receipt_hash = corrective_receipt_hash(CorrectiveReceiptHashInput {
        action_kind: draft.action_kind,
        store_label: store_label.as_str(),
        status: draft.status,
        issue_code: issue_code.as_deref(),
        evidence_hashes: evidence_hashes.as_slice(),
    });
    Ok(CorrectiveHostActionReceipt {
        receipt_hash,
        action_kind: draft.action_kind,
        store_label,
        status: draft.status,
        issue_code,
        evidence_hashes,
    })
}

fn corrective_denial_reason(
    record: &CorrectiveRecord,
    policy: &CorrectiveAdmissionPolicy<'_>,
) -> CorrectiveAdmissionReason {
    if record.inclusion() == CorrectiveInclusionPolicy::AlwaysRead {
        return CorrectiveAdmissionReason::AlwaysReadRejected;
    }
    if record.inclusion() == CorrectiveInclusionPolicy::Never {
        return CorrectiveAdmissionReason::InclusionDisabled;
    }
    if !policy.trusted_sources.contains(&record.source()) {
        return CorrectiveAdmissionReason::SourceNotTrusted;
    }
    if !policy.allowed_scopes.contains(&record.trust_scope()) {
        return CorrectiveAdmissionReason::ScopeDenied;
    }
    match record.freshness() {
        CorrectiveFreshness::Expired => return CorrectiveAdmissionReason::ExpiredRecord,
        CorrectiveFreshness::Stale if !policy.allow_stale => {
            return CorrectiveAdmissionReason::StaleRecord;
        }
        CorrectiveFreshness::Fresh | CorrectiveFreshness::Stale => {}
    }
    match record.redaction() {
        CorrectiveRedactionState::Secret => return CorrectiveAdmissionReason::SecretRedaction,
        CorrectiveRedactionState::MetadataOnly => return CorrectiveAdmissionReason::MetadataOnly,
        CorrectiveRedactionState::Public => {}
    }
    if record.prompt_budget_units() > policy.prompt_budget_units {
        return CorrectiveAdmissionReason::BudgetExceeded;
    }
    CorrectiveAdmissionReason::Accepted
}

fn validate_blake3_hash(field: &str, value: &str) -> Result<String, ReviewReportError> {
    let hash = validate_safe_metadata(field, value)?;
    if hash.len() == crate::SAFE_HASH_HEX_CHARS && hash.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        Ok(hash)
    } else {
        Err(ReviewReportError::new(
            ReviewDiagnosticCode::InvalidHash,
            field,
        ))
    }
}

fn promotion_target_from_route(route: &str) -> Result<CorrectivePromotionTarget, ReviewReportError> {
    match route {
        PROMOTION_TARGET_DETERMINISTIC_CHECK | PROMOTION_TARGET_CHECK_ALIAS => {
            Ok(CorrectivePromotionTarget::DeterministicCheck)
        }
        PROMOTION_TARGET_POLICY_GUIDANCE => Ok(CorrectivePromotionTarget::PolicyGuidance),
        PROMOTION_TARGET_SPEC_RULE => Ok(CorrectivePromotionTarget::SpecRule),
        PROMOTION_TARGET_MEMORY_GUIDANCE => Ok(CorrectivePromotionTarget::MemoryGuidance),
        PROMOTION_TARGET_HUMAN_CHECKPOINT => Ok(CorrectivePromotionTarget::HumanCheckpoint),
        _unsupported => Err(ReviewReportError::new(
            ReviewDiagnosticCode::UnsupportedPromotionTarget,
            TAG_ROUTE,
        )),
    }
}

fn validate_store_available(status: CorrectiveStoreStatus) -> Result<(), ReviewReportError> {
    match status {
        CorrectiveStoreStatus::Available => Ok(()),
        CorrectiveStoreStatus::Missing => Err(ReviewReportError::new(
            ReviewDiagnosticCode::MissingStore,
            "store_status",
        )),
        CorrectiveStoreStatus::Unavailable => Err(ReviewReportError::new(
            ReviewDiagnosticCode::UnavailableStore,
            "store_status",
        )),
    }
}

fn corrective_record_hash(record: &CorrectiveRecord) -> String {
    let mut canonical = String::from(CORRECTIVE_HASH_DOMAIN);
    push_corrective_hash_part(&mut canonical, record.record_id());
    push_corrective_hash_part(&mut canonical, record.kind().as_str());
    push_corrective_hash_part(&mut canonical, record.source());
    push_corrective_hash_part(&mut canonical, record.trust_scope().as_str());
    push_corrective_hash_part(&mut canonical, record.subject());
    push_corrective_hash_part(&mut canonical, record.class());
    push_corrective_hash_part(&mut canonical, record.scope());
    push_corrective_hash_part(&mut canonical, record.route());
    push_corrective_hash_part(&mut canonical, record.freshness().as_str());
    push_corrective_hash_part(&mut canonical, record.inclusion().as_str());
    push_corrective_hash_part(
        &mut canonical,
        format!("{}", record.prompt_budget_units()).as_str(),
    );
    push_corrective_hash_part(&mut canonical, record.body_hash());
    push_corrective_hash_part(&mut canonical, record.redaction().as_str());
    hash_text(canonical.as_str())
}

fn corrective_candidate_hash(candidate: &CorrectivePromotionCandidate) -> String {
    let mut canonical = String::from(CORRECTIVE_HASH_DOMAIN);
    push_corrective_hash_part(&mut canonical, candidate.class());
    push_corrective_hash_part(&mut canonical, candidate.scope());
    push_corrective_hash_part(&mut canonical, candidate.route());
    push_corrective_hash_part(&mut canonical, candidate.target().as_str());
    push_corrective_hash_part(&mut canonical, format!("{}", candidate.count()).as_str());
    for evidence_hash in candidate.evidence_hashes() {
        push_corrective_hash_part(&mut canonical, evidence_hash.as_str());
    }
    hash_text(canonical.as_str())
}

#[derive(Clone, Copy)]
struct CorrectiveReceiptHashInput<'a> {
    action_kind: CorrectiveHostActionKind,
    store_label: &'a str,
    status: CorrectiveHostActionReceiptStatus,
    issue_code: Option<&'a str>,
    evidence_hashes: &'a [String],
}

fn corrective_receipt_hash(input: CorrectiveReceiptHashInput<'_>) -> String {
    let mut canonical = String::from(CORRECTIVE_ACTION_HASH_DOMAIN);
    push_corrective_hash_part(&mut canonical, input.action_kind.as_str());
    push_corrective_hash_part(&mut canonical, input.store_label);
    push_corrective_hash_part(&mut canonical, input.status.as_str());
    push_corrective_hash_part(&mut canonical, input.issue_code.unwrap_or_default());
    for evidence_hash in input.evidence_hashes {
        push_corrective_hash_part(&mut canonical, evidence_hash.as_str());
    }
    hash_text(canonical.as_str())
}

fn push_corrective_hash_part(canonical: &mut String, part: &str) {
    canonical.push_str(HASH_SEPARATOR);
    canonical.push_str(part);
}
