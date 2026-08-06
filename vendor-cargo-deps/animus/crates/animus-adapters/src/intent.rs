//! Optional Turnstone-inspired intent validation contracts.
//!
//! Intent verdicts are evidence for host admission and operator review. They do
//! not grant authority and cannot bypass capability, policy, confirmation,
//! sandbox, mutation-window, or executor checks.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::assertion_density,
        tigerstyle::non_trait_imports,
        tigerstyle::path_segment_repetition
    )
)]

use crate::artifact::Hash;
use crate::validation::AdapterContractError;
use crate::validation::FieldName;
use crate::validation::{self};

const CALL_ID_FIELD: FieldName = FieldName::new("intent_call_id");
const BATCH_ID_FIELD: FieldName = FieldName::new("intent_batch_id");
const SUMMARY_LABEL_FIELD: FieldName = FieldName::new("intent_summary_label");
const EVIDENCE_LABEL_FIELD: FieldName = FieldName::new("intent_evidence_label");
const REVIEWER_LABEL_FIELD: FieldName = FieldName::new("intent_reviewer_label");
const RULE_ID_FIELD: FieldName = FieldName::new("intent_rule_id");
const RATIONALE_LABEL_FIELD: FieldName = FieldName::new("intent_rationale_label");
const EXAMPLE_FIELD: FieldName = FieldName::new("intent_example");
const TOOL_LABEL_FIELD: FieldName = FieldName::new("intent_tool_label");
const RESOURCE_LABEL_FIELD: FieldName = FieldName::new("intent_resource_label");
const RECEIPT_STATUS_FIELD: FieldName = FieldName::new("intent_receipt_status");
const RECEIPT_ISSUE_FIELD: FieldName = FieldName::new("intent_receipt_issue");
const HASH_FIELD: FieldName = FieldName::new("intent_hash");

const MIN_CONFIDENCE: u8 = 0;
const MAX_CONFIDENCE: u8 = 100;
const MIN_VERDICT_COUNT: u16 = 1;
const MAX_EVIDENCE_REFS: usize = 16;
const MAX_EXAMPLES: usize = 16;
const MAX_VERDICTS: usize = 32;
const STALE_VERDICT_EPOCH_DELTA: u32 = 3;
const SENSITIVE_ADJACENT_LABEL: &str = "sensitive-adjacent";
const NETWORK_OUTPUT_LABEL: &str = "network-output";
const UNSAFE_INTENT_MARKERS: &[&str] = &[
    "raw prompt",
    "provider payload",
    "credential",
    "secret=",
    "token=",
    "/home/",
    "/tmp/",
    "c:\\",
    "\\users\\",
    "raw command",
    "raw arguments",
    "bash -c",
    "rm -rf",
    "raw diff",
    "raw tool output",
    "unbounded transcript",
];

const _: () = assert!(MIN_VERDICT_COUNT > 0);
const _: () = assert!(MAX_EVIDENCE_REFS > 0);
const _: () = assert!(MAX_EXAMPLES > 0);
const _: () = assert!(MAX_VERDICTS > 0);
const _: () = assert!(STALE_VERDICT_EPOCH_DELTA > 0);

/// Product behaviors that intent validation explicitly does not perform.
pub const PROHIBITED_INTENT_VALIDATION_EFFECTS: &[&str] = &[
    "smart approval",
    "live LLM judge execution",
    "policy bypass",
    "capability grant",
    "confirmation suppression",
    "sandbox bypass",
    "tool execution",
];

/// Tool-call action class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntentActionClass {
    /// Read-only information access.
    ReadOnly,
    /// Mutation of files, state, or lifecycle artifacts.
    Mutation,
    /// Process or command execution.
    Process,
    /// Network access or output.
    Network,
    /// Secret-adjacent resource access.
    SecretAdjacent,
    /// Persistent control-plane change.
    ControlPlane,
}

/// Rule match class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntentMatchClass {
    /// Match by tool kind.
    ToolKind,
    /// Match by command fingerprint or class.
    CommandPattern,
    /// Match by resource class.
    ResourceClass,
    /// Match by batch interaction.
    BatchInteraction,
}

/// Risk level.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IntentRiskLevel {
    /// Low risk.
    Low,
    /// Medium risk.
    Medium,
    /// High risk.
    High,
    /// Critical risk.
    Critical,
}

/// Recommendation class.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IntentRecommendation {
    /// Allow if all normal gates also pass.
    Allow,
    /// Require operator/host review.
    RequireReview,
    /// Deny before dispatch.
    Deny,
    /// Add advisory metadata only.
    Annotate,
}

/// Detector tier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntentDetectorTier {
    /// Deterministic heuristic tier.
    Deterministic,
    /// Optional reviewer/classifier tier.
    Reviewer,
    /// Batch interaction tier.
    Batch,
}

/// Latency bucket for verdict production.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntentLatencyBucket {
    /// Immediate deterministic result.
    Immediate,
    /// Fast asynchronous result.
    Fast,
    /// Slow result.
    Slow,
    /// Pending result.
    Pending,
}

/// Batch admission status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntentBatchAdmissionStatus {
    /// Batch may proceed if normal gates also pass.
    Allowed,
    /// Batch requires review before any call executes.
    ReviewRequired,
    /// Batch is denied.
    Denied,
}

/// Intent diagnostic code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntentDiagnosticCode {
    /// Required field is empty.
    EmptyField,
    /// Field is too long.
    FieldTooLong,
    /// Field contains a NUL byte.
    NulCharacter,
    /// Collection is empty.
    EmptyCollection,
    /// Collection is too large.
    TooManyItems,
    /// Text or metadata leaked unsafe content.
    UnsafeMetadata,
    /// Evidence hash is malformed.
    InvalidHash,
    /// Confidence is outside bounds.
    InvalidConfidence,
    /// Verdict is stale.
    StaleVerdict,
    /// Evidence is missing.
    MissingEvidence,
    /// Rule examples do not match declared class.
    MalformedRuleFixture,
}

/// Safe intent diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntentDiagnostic {
    code: IntentDiagnosticCode,
    subject_id: Option<Text>,
    message: Text,
}

/// Intent verdict draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntentVerdictDraft<'a> {
    /// Tool-call identity.
    pub call_id: &'a str,
    /// Batch identity.
    pub batch_id: &'a str,
    /// Safe intent summary label.
    pub summary_label: &'a str,
    /// Risk level.
    pub risk: IntentRiskLevel,
    /// Confidence score.
    pub confidence: u8,
    /// Recommendation.
    pub recommendation: IntentRecommendation,
    /// Detector tier.
    pub tier: IntentDetectorTier,
    /// Evidence labels.
    pub evidence_labels: &'a [&'a str],
    /// Evidence hashes.
    pub evidence_hashes: &'a [&'a str],
    /// Latency bucket.
    pub latency: IntentLatencyBucket,
    /// Optional reviewer label.
    pub reviewer_label: Option<&'a str>,
    /// Observation epoch.
    pub observed_epoch: u32,
    /// Current host epoch.
    pub current_epoch: u32,
}

/// Validated intent verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntentVerdict {
    call_id: Text,
    batch_id: Text,
    summary_label: Text,
    risk: IntentRiskLevel,
    confidence: u8,
    recommendation: IntentRecommendation,
    tier: IntentDetectorTier,
    evidence_labels: Collection<Text>,
    evidence_hashes: Collection<Text>,
    latency: IntentLatencyBucket,
    reviewer_label: Option<Text>,
}

/// Heuristic rule draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntentHeuristicRuleDraft<'a> {
    /// Stable rule id.
    pub rule_id: &'a str,
    /// Action class.
    pub action_class: IntentActionClass,
    /// Match class.
    pub match_class: IntentMatchClass,
    /// Risk level.
    pub risk: IntentRiskLevel,
    /// Recommendation.
    pub recommendation: IntentRecommendation,
    /// Safe rationale label.
    pub rationale_label: &'a str,
    /// Positive fixture examples.
    pub positive_examples: &'a [&'a str],
    /// Negative fixture examples.
    pub negative_examples: &'a [&'a str],
}

/// Validated heuristic rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntentHeuristicRule {
    rule_id: Text,
    action_class: IntentActionClass,
    match_class: IntentMatchClass,
    risk: IntentRiskLevel,
    recommendation: IntentRecommendation,
    rationale_label: Text,
    positive_examples: Collection<Text>,
    negative_examples: Collection<Text>,
}

/// Pending tool call summary supplied by host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingIntentCallDraft<'a> {
    /// Tool-call identity.
    pub call_id: &'a str,
    /// Batch identity.
    pub batch_id: &'a str,
    /// Safe tool label.
    pub tool_label: &'a str,
    /// Action class.
    pub action_class: IntentActionClass,
    /// Resource class label.
    pub resource_label: &'a str,
    /// Evidence hash.
    pub evidence_hash: &'a str,
}

/// Validated pending call summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingIntentCall {
    call_id: Text,
    batch_id: Text,
    tool_label: Text,
    action_class: IntentActionClass,
    resource_label: Text,
    evidence_hash: Text,
}

/// Batch admission result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntentBatchAdmission {
    status: IntentBatchAdmissionStatus,
    batch_id: Text,
    reason: Option<Text>,
}

/// Intent receipt draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntentReceiptDraft<'a> {
    /// Batch identity.
    pub batch_id: &'a str,
    /// Receipt status label.
    pub status: &'a str,
    /// Verdict count.
    pub verdict_count: u16,
    /// Policy hash.
    pub policy_hash: &'a str,
    /// Evidence hash.
    pub evidence_hash: &'a str,
    /// Redaction marker.
    pub redaction_marker: &'a str,
    /// Recommendation class.
    pub recommendation: IntentRecommendation,
    /// Optional safe issue code.
    pub issue_code: Option<&'a str>,
}

/// Validated safe receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntentReceipt {
    batch_id: Text,
    status: Text,
    verdict_count: u16,
    policy_hash: Text,
    evidence_hash: Text,
    redaction_marker: Text,
    recommendation: IntentRecommendation,
    issue_code: Option<Text>,
}

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;
type IntentResult<T> = Result<T, IntentDiagnostic>;

impl IntentDiagnostic {
    /// Diagnostic code.
    #[must_use]
    pub const fn code(&self) -> IntentDiagnosticCode {
        self.code
    }

    /// Subject id.
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

impl IntentVerdict {
    /// Call identity.
    #[must_use]
    pub fn call_id(&self) -> &str {
        &self.call_id
    }

    /// Batch identity.
    #[must_use]
    pub fn batch_id(&self) -> &str {
        &self.batch_id
    }

    /// Intent summary label.
    #[must_use]
    pub fn summary_label(&self) -> &str {
        &self.summary_label
    }

    /// Risk level.
    #[must_use]
    pub const fn risk(&self) -> IntentRiskLevel {
        self.risk
    }

    /// Confidence.
    #[must_use]
    pub const fn confidence(&self) -> u8 {
        self.confidence
    }

    /// Recommendation.
    #[must_use]
    pub const fn recommendation(&self) -> IntentRecommendation {
        self.recommendation
    }

    /// Detector tier.
    #[must_use]
    pub const fn tier(&self) -> IntentDetectorTier {
        self.tier
    }

    /// Evidence labels.
    #[must_use]
    pub fn evidence_labels(&self) -> &[Text] {
        &self.evidence_labels
    }

    /// Evidence hashes.
    #[must_use]
    pub fn evidence_hashes(&self) -> &[Text] {
        &self.evidence_hashes
    }

    /// Latency bucket.
    #[must_use]
    pub const fn latency(&self) -> IntentLatencyBucket {
        self.latency
    }

    /// Reviewer label.
    #[must_use]
    pub fn reviewer_label(&self) -> Option<&str> {
        self.reviewer_label.as_deref()
    }
}

impl IntentHeuristicRule {
    /// Rule id.
    #[must_use]
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    /// Action class.
    #[must_use]
    pub const fn action_class(&self) -> IntentActionClass {
        self.action_class
    }

    /// Match class.
    #[must_use]
    pub const fn match_class(&self) -> IntentMatchClass {
        self.match_class
    }

    /// Risk level.
    #[must_use]
    pub const fn risk(&self) -> IntentRiskLevel {
        self.risk
    }

    /// Recommendation.
    #[must_use]
    pub const fn recommendation(&self) -> IntentRecommendation {
        self.recommendation
    }

    /// Rationale label.
    #[must_use]
    pub fn rationale_label(&self) -> &str {
        &self.rationale_label
    }

    /// Positive examples.
    #[must_use]
    pub fn positive_examples(&self) -> &[Text] {
        &self.positive_examples
    }

    /// Negative examples.
    #[must_use]
    pub fn negative_examples(&self) -> &[Text] {
        &self.negative_examples
    }
}

impl PendingIntentCall {
    /// Call identity.
    #[must_use]
    pub fn call_id(&self) -> &str {
        &self.call_id
    }

    /// Batch identity.
    #[must_use]
    pub fn batch_id(&self) -> &str {
        &self.batch_id
    }

    /// Tool label.
    #[must_use]
    pub fn tool_label(&self) -> &str {
        &self.tool_label
    }

    /// Action class.
    #[must_use]
    pub const fn action_class(&self) -> IntentActionClass {
        self.action_class
    }

    /// Resource label.
    #[must_use]
    pub fn resource_label(&self) -> &str {
        &self.resource_label
    }

    /// Evidence hash.
    #[must_use]
    pub fn evidence_hash(&self) -> &str {
        &self.evidence_hash
    }
}

impl IntentBatchAdmission {
    /// Batch status.
    #[must_use]
    pub const fn status(&self) -> IntentBatchAdmissionStatus {
        self.status
    }

    /// Batch id.
    #[must_use]
    pub fn batch_id(&self) -> &str {
        &self.batch_id
    }

    /// Safe reason.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}

impl IntentReceipt {
    /// Batch identity.
    #[must_use]
    pub fn batch_id(&self) -> &str {
        &self.batch_id
    }

    /// Receipt status.
    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Verdict count.
    #[must_use]
    pub const fn verdict_count(&self) -> u16 {
        self.verdict_count
    }

    /// Policy hash.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        &self.policy_hash
    }

    /// Evidence hash.
    #[must_use]
    pub fn evidence_hash(&self) -> &str {
        &self.evidence_hash
    }

    /// Redaction marker.
    #[must_use]
    pub fn redaction_marker(&self) -> &str {
        &self.redaction_marker
    }

    /// Recommendation.
    #[must_use]
    pub const fn recommendation(&self) -> IntentRecommendation {
        self.recommendation
    }

    /// Issue code.
    #[must_use]
    pub fn issue_code(&self) -> Option<&str> {
        self.issue_code.as_deref()
    }
}

/// Validates an intent verdict DTO.
///
/// # Errors
///
/// Returns [`IntentDiagnostic`] for unsafe metadata, malformed hashes, missing
/// evidence, stale verdicts, or out-of-range confidence.
pub fn validate_intent_verdict(draft: &IntentVerdictDraft<'_>) -> IntentResult<IntentVerdict> {
    validate_confidence(draft.confidence)?;
    if is_stale(draft.observed_epoch, draft.current_epoch) {
        return Err(diagnostic(IntentDiagnosticCode::StaleVerdict, Some(draft.call_id), "intent verdict is stale"));
    }
    let call_id = safe_text(CALL_ID_FIELD, draft.call_id, None)?;
    let batch_id = safe_text(BATCH_ID_FIELD, draft.batch_id, Some(call_id.as_str()))?;
    if draft.evidence_labels.is_empty() || draft.evidence_hashes.is_empty() {
        return Err(diagnostic(
            IntentDiagnosticCode::MissingEvidence,
            Some(call_id.as_str()),
            "intent verdict evidence is missing",
        ));
    }
    let evidence_labels = validate_text_list(
        "intent_evidence_labels",
        draft.evidence_labels,
        EVIDENCE_LABEL_FIELD,
        Some(call_id.as_str()),
    )?;
    let evidence_hashes = validate_hashes(draft.evidence_hashes)?;
    let reviewer_label = draft
        .reviewer_label
        .map(|label| safe_text(REVIEWER_LABEL_FIELD, label, Some(call_id.as_str())))
        .transpose()?;
    Ok(IntentVerdict {
        call_id,
        batch_id,
        summary_label: safe_text(SUMMARY_LABEL_FIELD, draft.summary_label, None)?,
        risk: draft.risk,
        confidence: draft.confidence,
        recommendation: draft.recommendation,
        tier: draft.tier,
        evidence_labels,
        evidence_hashes,
        latency: draft.latency,
        reviewer_label,
    })
}

/// Validates a deterministic heuristic rule and its fixtures.
///
/// # Errors
///
/// Returns [`IntentDiagnostic`] when examples are missing, unsafe, or do not
/// match the declared action class fixture convention.
pub fn validate_intent_heuristic_rule(draft: &IntentHeuristicRuleDraft<'_>) -> IntentResult<IntentHeuristicRule> {
    validate_non_empty_bound("intent_positive_examples", draft.positive_examples.len(), MAX_EXAMPLES)?;
    validate_non_empty_bound("intent_negative_examples", draft.negative_examples.len(), MAX_EXAMPLES)?;
    let rule_id = safe_text(RULE_ID_FIELD, draft.rule_id, None)?;
    let rationale_label = safe_text(RATIONALE_LABEL_FIELD, draft.rationale_label, Some(rule_id.as_str()))?;
    let positive_examples =
        validate_text_list("intent_positive_examples", draft.positive_examples, EXAMPLE_FIELD, Some(rule_id.as_str()))?;
    let negative_examples =
        validate_text_list("intent_negative_examples", draft.negative_examples, EXAMPLE_FIELD, Some(rule_id.as_str()))?;
    if !positive_examples.iter().all(|example| example_matches_action_class(example, draft.action_class))
        || negative_examples.iter().any(|example| example_matches_action_class(example, draft.action_class))
    {
        return Err(diagnostic(
            IntentDiagnosticCode::MalformedRuleFixture,
            Some(rule_id.as_str()),
            "heuristic examples do not match action class",
        ));
    }
    Ok(IntentHeuristicRule {
        rule_id,
        action_class: draft.action_class,
        match_class: draft.match_class,
        risk: draft.risk,
        recommendation: draft.recommendation,
        rationale_label,
        positive_examples,
        negative_examples,
    })
}

/// Validates a pending call summary.
///
/// # Errors
///
/// Returns [`IntentDiagnostic`] when safe labels or evidence identities are invalid.
pub fn validate_pending_intent_call(draft: &PendingIntentCallDraft<'_>) -> IntentResult<PendingIntentCall> {
    let call_id = safe_text(CALL_ID_FIELD, draft.call_id, None)?;
    Ok(PendingIntentCall {
        call_id: call_id.clone(),
        batch_id: safe_text(BATCH_ID_FIELD, draft.batch_id, Some(call_id.as_str()))?,
        tool_label: safe_text(TOOL_LABEL_FIELD, draft.tool_label, Some(call_id.as_str()))?,
        action_class: draft.action_class,
        resource_label: safe_text(RESOURCE_LABEL_FIELD, draft.resource_label, Some(call_id.as_str()))?,
        evidence_hash: safe_hash(draft.evidence_hash)?,
    })
}

/// Evaluates deterministic intent heuristics over supplied data only.
#[must_use]
pub fn evaluate_intent_heuristics(call: &PendingIntentCall, rules: &[IntentHeuristicRule]) -> IntentVerdict {
    let selected = rules.iter().find(|rule| rule.action_class() == call.action_class());
    let (risk, recommendation, summary) = match selected {
        Some(rule) => (rule.risk(), rule.recommendation(), rule.rationale_label()),
        None => (IntentRiskLevel::Low, IntentRecommendation::Allow, "no-matching-risk-rule"),
    };
    IntentVerdict {
        call_id: Text::from(call.call_id()),
        batch_id: Text::from(call.batch_id()),
        summary_label: Text::from(summary),
        risk,
        confidence: MAX_CONFIDENCE,
        recommendation,
        tier: IntentDetectorTier::Deterministic,
        evidence_labels: collection_with_one(Text::from(call.resource_label())),
        evidence_hashes: collection_with_one(Text::from(call.evidence_hash())),
        latency: IntentLatencyBucket::Immediate,
        reviewer_label: None,
    }
}

/// Merges optional reviewer output with deterministic verdicts using tighten-only semantics.
#[must_use]
pub fn merge_intent_reviewer_verdict(
    deterministic: &IntentVerdict,
    reviewer: Option<&IntentVerdict>,
    authority_present: bool,
    confirmation_required: bool,
) -> IntentVerdict {
    let mut merged = deterministic.clone();
    if let Some(reviewer) = reviewer {
        merged.risk = core::cmp::max(merged.risk(), reviewer.risk());
        merged.recommendation = tighter_recommendation(merged.recommendation(), reviewer.recommendation());
        merged.reviewer_label = reviewer.reviewer_label().map(Text::from);
    }
    if !authority_present || confirmation_required {
        merged.recommendation = tighter_recommendation(merged.recommendation(), IntentRecommendation::RequireReview);
    }
    if deterministic.risk() == IntentRiskLevel::Critical || deterministic.recommendation() == IntentRecommendation::Deny
    {
        merged.risk = core::cmp::max(merged.risk(), IntentRiskLevel::Critical);
        merged.recommendation = IntentRecommendation::Deny;
    }
    merged
}

/// Evaluates batch-level risk before any partial dispatch.
#[must_use]
pub fn evaluate_intent_batch(verdicts: &[IntentVerdict]) -> IntentBatchAdmission {
    let batch_id = verdicts.first().map_or(Text::from("empty-batch"), |verdict| Text::from(verdict.batch_id()));
    if verdicts.iter().any(|verdict| verdict.recommendation() == IntentRecommendation::Deny) {
        return IntentBatchAdmission {
            status: IntentBatchAdmissionStatus::Denied,
            batch_id,
            reason: Some(Text::from("denied-verdict-present")),
        };
    }
    let has_sensitive_read = verdicts
        .iter()
        .any(|verdict| verdict.evidence_labels().iter().any(|label| label.contains(SENSITIVE_ADJACENT_LABEL)));
    let has_network_output = verdicts.iter().any(|verdict| {
        verdict.evidence_labels().iter().any(|label| label.contains(NETWORK_OUTPUT_LABEL))
            || verdict.summary_label().contains(NETWORK_OUTPUT_LABEL)
    });
    if has_sensitive_read && has_network_output {
        return IntentBatchAdmission {
            status: IntentBatchAdmissionStatus::ReviewRequired,
            batch_id,
            reason: Some(Text::from("sensitive-read-network-interaction")),
        };
    }
    if verdicts.iter().any(|verdict| verdict.recommendation() == IntentRecommendation::RequireReview) {
        return IntentBatchAdmission {
            status: IntentBatchAdmissionStatus::ReviewRequired,
            batch_id,
            reason: Some(Text::from("review-required-verdict-present")),
        };
    }
    IntentBatchAdmission {
        status: IntentBatchAdmissionStatus::Allowed,
        batch_id,
        reason: None,
    }
}

/// Validates safe intent-validation receipt metadata.
///
/// # Errors
///
/// Returns [`IntentDiagnostic`] when receipt fields leak raw command/tool bodies or hashes are
/// malformed.
pub fn validate_intent_receipt(draft: &IntentReceiptDraft<'_>) -> IntentResult<IntentReceipt> {
    let batch_id = safe_text(BATCH_ID_FIELD, draft.batch_id, None)?;
    if draft.verdict_count < MIN_VERDICT_COUNT || usize::from(draft.verdict_count) > MAX_VERDICTS {
        return Err(diagnostic(
            IntentDiagnosticCode::TooManyItems,
            Some(batch_id.as_str()),
            "intent verdict count outside bounds",
        ));
    }
    let issue_code = draft
        .issue_code
        .map(|issue| safe_text(RECEIPT_ISSUE_FIELD, issue, Some(batch_id.as_str())))
        .transpose()?;
    Ok(IntentReceipt {
        batch_id,
        status: safe_text(RECEIPT_STATUS_FIELD, draft.status, None)?,
        verdict_count: draft.verdict_count,
        policy_hash: safe_hash(draft.policy_hash)?,
        evidence_hash: safe_hash(draft.evidence_hash)?,
        redaction_marker: safe_text(EVIDENCE_LABEL_FIELD, draft.redaction_marker, None)?,
        recommendation: draft.recommendation,
        issue_code,
    })
}

fn tighter_recommendation(left: IntentRecommendation, right: IntentRecommendation) -> IntentRecommendation {
    use IntentRecommendation::Allow;
    use IntentRecommendation::Annotate;
    use IntentRecommendation::Deny;
    use IntentRecommendation::RequireReview;
    match (left, right) {
        (Deny, _) | (_, Deny) => Deny,
        (RequireReview, _) | (_, RequireReview) => RequireReview,
        (Annotate, _) | (_, Annotate) => Annotate,
        (Allow, Allow) => Allow,
    }
}

fn collection_with_one<T>(item: T) -> Collection<T> {
    let mut values = Collection::with_capacity(usize::from(MIN_VERDICT_COUNT));
    values.push(item);
    values
}

fn validate_confidence(confidence: u8) -> IntentResult<()> {
    if !(MIN_CONFIDENCE..=MAX_CONFIDENCE).contains(&confidence) {
        return Err(diagnostic(IntentDiagnosticCode::InvalidConfidence, None, "intent confidence outside bounds"));
    }
    Ok(())
}

fn validate_text_list(
    field_name: &'static str,
    values: &[&str],
    item_field: FieldName,
    subject_id: Option<&str>,
) -> IntentResult<Collection<Text>> {
    validate_non_empty_bound(field_name, values.len(), MAX_EVIDENCE_REFS)?;
    let mut result = Collection::with_capacity(values.len());
    for value in values {
        result.push(safe_text(item_field, value, subject_id)?);
    }
    Ok(result)
}

fn validate_hashes(values: &[&str]) -> IntentResult<Collection<Text>> {
    validate_non_empty_bound("intent_evidence_hashes", values.len(), MAX_EVIDENCE_REFS)?;
    let mut result = Collection::with_capacity(values.len());
    for value in values {
        result.push(safe_hash(value)?);
    }
    Ok(result)
}

fn validate_non_empty_bound(field: &'static str, len: usize, max: usize) -> IntentResult<()> {
    if len == 0 {
        return Err(IntentDiagnostic {
            code: IntentDiagnosticCode::EmptyCollection,
            subject_id: None,
            message: Text::from(field),
        });
    }
    if len > max {
        return Err(IntentDiagnostic {
            code: IntentDiagnosticCode::TooManyItems,
            subject_id: None,
            message: Text::from(field),
        });
    }
    Ok(())
}

fn safe_text(field: FieldName, value: &str, subject_id: Option<&str>) -> IntentResult<Text> {
    let normalized = validation::normalized_adapter_field(field, value).map_err(|error| map_adapter_error(&error))?;
    let lower = normalized.to_ascii_lowercase();
    if UNSAFE_INTENT_MARKERS.iter().any(|marker| lower.contains(marker)) {
        return Err(diagnostic(
            IntentDiagnosticCode::UnsafeMetadata,
            subject_id,
            "intent metadata contains unsafe marker",
        ));
    }
    Ok(normalized)
}

fn safe_hash(value: &str) -> IntentResult<Text> {
    let normalized =
        validation::normalized_adapter_field(HASH_FIELD, value).map_err(|error| map_adapter_error(&error))?;
    if Hash::new(&normalized).is_ok() {
        Ok(normalized)
    } else {
        Err(diagnostic(IntentDiagnosticCode::InvalidHash, None, HASH_FIELD.as_str()))
    }
}

fn is_stale(observed_epoch: u32, current_epoch: u32) -> bool {
    current_epoch.saturating_sub(observed_epoch) > STALE_VERDICT_EPOCH_DELTA
}

fn example_matches_action_class(example: &str, class: IntentActionClass) -> bool {
    let lower = example.to_ascii_lowercase();
    match class {
        IntentActionClass::ReadOnly => lower.contains("read"),
        IntentActionClass::Mutation => lower.contains("mutation"),
        IntentActionClass::Process => lower.contains("process"),
        IntentActionClass::Network => lower.contains("network"),
        IntentActionClass::SecretAdjacent => lower.contains("secret-adjacent"),
        IntentActionClass::ControlPlane => lower.contains("control-plane"),
    }
}

fn map_adapter_error(error: &AdapterContractError) -> IntentDiagnostic {
    match error {
        AdapterContractError::EmptyField { field } => diagnostic(IntentDiagnosticCode::EmptyField, None, field),
        AdapterContractError::FieldTooLong { field, .. } => diagnostic(IntentDiagnosticCode::FieldTooLong, None, field),
        AdapterContractError::NulCharacter { field, .. } => diagnostic(IntentDiagnosticCode::NulCharacter, None, field),
        AdapterContractError::InvalidEvidenceHash { field, .. } => {
            diagnostic(IntentDiagnosticCode::InvalidHash, None, field)
        }
        AdapterContractError::EmptyCollection { field } => {
            diagnostic(IntentDiagnosticCode::EmptyCollection, None, field)
        }
        AdapterContractError::UnsupportedStatusState { field, .. }
        | AdapterContractError::UnsupportedStatusObservationKind { field, .. }
        | AdapterContractError::UnsupportedSessionReferenceKind { field, .. }
        | AdapterContractError::UnsupportedWorkspaceValue { field, .. }
        | AdapterContractError::UnsupportedDataspaceValue { field, .. }
        | AdapterContractError::UnsupportedRouterValue { field, .. } => {
            diagnostic(IntentDiagnosticCode::UnsafeMetadata, None, field)
        }
    }
}

fn diagnostic(code: IntentDiagnosticCode, subject_id: Option<&str>, message: &str) -> IntentDiagnostic {
    IntentDiagnostic {
        code,
        subject_id: subject_id.map(Text::from),
        message: Text::from(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CALL_READ: &str = "call-read";
    const CALL_MUTATE: &str = "call-mutate";
    const CALL_PROCESS: &str = "call-process";
    const CALL_NETWORK: &str = "call-network";
    const BATCH_ID: &str = "batch-alpha";
    const TOOL_READ: &str = "read-tool";
    const TOOL_WRITE: &str = "write-tool";
    const RESOURCE_READ: &str = "workspace-summary";
    const RESOURCE_SENSITIVE: &str = "sensitive-adjacent-config";
    const SUMMARY_READ: &str = "low-risk-read";
    const SUMMARY_PROCESS: &str = "critical-process-command";
    const RULE_READ: &str = "rule-read";
    const RULE_MUTATION: &str = "rule-mutation";
    const RULE_PROCESS: &str = "rule-process";
    const RATIONALE_READ: &str = "read-only-intent";
    const RATIONALE_MUTATION: &str = "mutation-review-intent";
    const RATIONALE_PROCESS: &str = "critical-process-intent";
    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const SINGLE_VERDICT_COUNT: u16 = 1;
    const CURRENT_EPOCH: u32 = 7;
    const FRESH_EPOCH: u32 = 7;
    const STALE_EPOCH: u32 = 1;
    const CONFIDENCE_HIGH: u8 = 90;
    const CONFIDENCE_INVALID: u8 = 101;
    static EVIDENCE_LABELS_READ: [&str; 1] = [RESOURCE_READ];
    static EVIDENCE_LABELS_SENSITIVE: [&str; 1] = [RESOURCE_SENSITIVE];
    static EVIDENCE_HASHES: [&str; 1] = [HASH_A];
    static READ_POSITIVE: [&str; 1] = ["read metadata fingerprint"];
    static READ_NEGATIVE: [&str; 1] = ["mutation metadata fingerprint"];
    static MUTATION_POSITIVE: [&str; 1] = ["mutation write fingerprint"];
    static MUTATION_NEGATIVE: [&str; 1] = ["read metadata fingerprint"];
    static PROCESS_POSITIVE: [&str; 1] = ["process destructive fingerprint"];
    static PROCESS_NEGATIVE: [&str; 1] = ["read metadata fingerprint"];

    fn verdict_draft(
        call_id: &'static str,
        risk: IntentRiskLevel,
        recommendation: IntentRecommendation,
    ) -> IntentVerdictDraft<'static> {
        IntentVerdictDraft {
            call_id,
            batch_id: BATCH_ID,
            summary_label: SUMMARY_READ,
            risk,
            confidence: CONFIDENCE_HIGH,
            recommendation,
            tier: IntentDetectorTier::Deterministic,
            evidence_labels: &EVIDENCE_LABELS_READ,
            evidence_hashes: &EVIDENCE_HASHES,
            latency: IntentLatencyBucket::Immediate,
            reviewer_label: None,
            observed_epoch: FRESH_EPOCH,
            current_epoch: CURRENT_EPOCH,
        }
    }

    #[derive(Clone, Copy)]
    struct RuleFixture {
        rule_id: &'static str,
        action_class: IntentActionClass,
        risk: IntentRiskLevel,
        recommendation: IntentRecommendation,
        rationale: &'static str,
        positive_examples: &'static [&'static str],
        negative_examples: &'static [&'static str],
    }

    fn rule(fixture: RuleFixture) -> IntentHeuristicRule {
        validate_intent_heuristic_rule(&IntentHeuristicRuleDraft {
            rule_id: fixture.rule_id,
            action_class: fixture.action_class,
            match_class: IntentMatchClass::ToolKind,
            risk: fixture.risk,
            recommendation: fixture.recommendation,
            rationale_label: fixture.rationale,
            positive_examples: fixture.positive_examples,
            negative_examples: fixture.negative_examples,
        })
        .unwrap()
    }

    fn rules() -> [IntentHeuristicRule; 3] {
        [
            rule(RuleFixture {
                rule_id: RULE_READ,
                action_class: IntentActionClass::ReadOnly,
                risk: IntentRiskLevel::Low,
                recommendation: IntentRecommendation::Allow,
                rationale: RATIONALE_READ,
                positive_examples: &READ_POSITIVE,
                negative_examples: &READ_NEGATIVE,
            }),
            rule(RuleFixture {
                rule_id: RULE_MUTATION,
                action_class: IntentActionClass::Mutation,
                risk: IntentRiskLevel::High,
                recommendation: IntentRecommendation::RequireReview,
                rationale: RATIONALE_MUTATION,
                positive_examples: &MUTATION_POSITIVE,
                negative_examples: &MUTATION_NEGATIVE,
            }),
            rule(RuleFixture {
                rule_id: RULE_PROCESS,
                action_class: IntentActionClass::Process,
                risk: IntentRiskLevel::Critical,
                recommendation: IntentRecommendation::Deny,
                rationale: RATIONALE_PROCESS,
                positive_examples: &PROCESS_POSITIVE,
                negative_examples: &PROCESS_NEGATIVE,
            }),
        ]
    }

    fn pending(
        call_id: &'static str,
        tool: &'static str,
        action_class: IntentActionClass,
        resource: &'static str,
    ) -> PendingIntentCall {
        validate_pending_intent_call(&PendingIntentCallDraft {
            call_id,
            batch_id: BATCH_ID,
            tool_label: tool,
            action_class,
            resource_label: resource,
            evidence_hash: HASH_A,
        })
        .unwrap()
    }

    #[test]
    fn read_only_verdict_dto_validates_safe_metadata() {
        // r[verify agent_harness_runtime.turnstone_intent_validation.dto]
        let verdict =
            validate_intent_verdict(&verdict_draft(CALL_READ, IntentRiskLevel::Low, IntentRecommendation::Allow))
                .unwrap();

        assert_eq!(verdict.call_id(), CALL_READ);
        assert_eq!(verdict.risk(), IntentRiskLevel::Low);
        assert_eq!(verdict.recommendation(), IntentRecommendation::Allow);
        assert_eq!(verdict.evidence_hashes().len(), 1);
        assert!(PROHIBITED_INTENT_VALIDATION_EFFECTS.contains(&"smart approval"));
    }

    #[test]
    fn heuristic_rules_validate_fixture_direction() {
        // r[verify agent_harness_runtime.turnstone_intent_validation.heuristics]
        let process = rule(RuleFixture {
            rule_id: RULE_PROCESS,
            action_class: IntentActionClass::Process,
            risk: IntentRiskLevel::Critical,
            recommendation: IntentRecommendation::Deny,
            rationale: RATIONALE_PROCESS,
            positive_examples: &PROCESS_POSITIVE,
            negative_examples: &PROCESS_NEGATIVE,
        });
        let malformed = validate_intent_heuristic_rule(&IntentHeuristicRuleDraft {
            positive_examples: &READ_POSITIVE,
            ..IntentHeuristicRuleDraft {
                rule_id: RULE_PROCESS,
                action_class: IntentActionClass::Process,
                match_class: IntentMatchClass::CommandPattern,
                risk: IntentRiskLevel::Critical,
                recommendation: IntentRecommendation::Deny,
                rationale_label: RATIONALE_PROCESS,
                positive_examples: &PROCESS_POSITIVE,
                negative_examples: &PROCESS_NEGATIVE,
            }
        })
        .expect_err("positive examples must match action class");

        assert_eq!(process.action_class(), IntentActionClass::Process);
        assert_eq!(malformed.code(), IntentDiagnosticCode::MalformedRuleFixture);
    }

    #[test]
    fn deterministic_heuristics_classify_read_mutation_and_process_calls() {
        // r[verify agent_harness_runtime.turnstone_intent_validation.evaluate]
        let rules = rules();
        let read = evaluate_intent_heuristics(
            &pending(CALL_READ, TOOL_READ, IntentActionClass::ReadOnly, RESOURCE_READ),
            &rules,
        );
        let mutation = evaluate_intent_heuristics(
            &pending(CALL_MUTATE, TOOL_WRITE, IntentActionClass::Mutation, RESOURCE_READ),
            &rules,
        );
        let process = evaluate_intent_heuristics(
            &pending(CALL_PROCESS, "process-tool", IntentActionClass::Process, RESOURCE_READ),
            &rules,
        );

        assert_eq!(read.recommendation(), IntentRecommendation::Allow);
        assert_eq!(mutation.recommendation(), IntentRecommendation::RequireReview);
        assert_eq!(process.recommendation(), IntentRecommendation::Deny);
    }

    #[test]
    fn reviewer_merge_cannot_override_deny_or_missing_authority() {
        // r[verify agent_harness_runtime.turnstone_intent_validation.reviewer]
        let deterministic = validate_intent_verdict(&IntentVerdictDraft {
            risk: IntentRiskLevel::Critical,
            recommendation: IntentRecommendation::Deny,
            summary_label: SUMMARY_PROCESS,
            ..verdict_draft(CALL_PROCESS, IntentRiskLevel::Critical, IntentRecommendation::Deny)
        })
        .unwrap();
        let reviewer_allow = validate_intent_verdict(&IntentVerdictDraft {
            tier: IntentDetectorTier::Reviewer,
            reviewer_label: Some("classifier-alpha"),
            recommendation: IntentRecommendation::Allow,
            risk: IntentRiskLevel::Low,
            ..verdict_draft(CALL_PROCESS, IntentRiskLevel::Low, IntentRecommendation::Allow)
        })
        .unwrap();
        let merged = merge_intent_reviewer_verdict(&deterministic, Some(&reviewer_allow), true, false);
        let missing_authority = merge_intent_reviewer_verdict(
            &validate_intent_verdict(&verdict_draft(CALL_READ, IntentRiskLevel::Low, IntentRecommendation::Allow))
                .unwrap(),
            Some(&reviewer_allow),
            false,
            false,
        );

        assert_eq!(merged.recommendation(), IntentRecommendation::Deny);
        assert_eq!(merged.risk(), IntentRiskLevel::Critical);
        assert_eq!(missing_authority.recommendation(), IntentRecommendation::RequireReview);
    }

    #[test]
    fn batch_interactions_require_review_before_partial_dispatch() {
        // r[verify agent_harness_runtime.turnstone_intent_validation.batch]
        let sensitive = validate_intent_verdict(&IntentVerdictDraft {
            call_id: CALL_READ,
            evidence_labels: &EVIDENCE_LABELS_SENSITIVE,
            ..verdict_draft(CALL_READ, IntentRiskLevel::Low, IntentRecommendation::Allow)
        })
        .unwrap();
        let network = validate_intent_verdict(&IntentVerdictDraft {
            call_id: CALL_NETWORK,
            summary_label: NETWORK_OUTPUT_LABEL,
            ..verdict_draft(CALL_NETWORK, IntentRiskLevel::Low, IntentRecommendation::Allow)
        })
        .unwrap();
        let admission = evaluate_intent_batch(&[sensitive, network]);

        assert_eq!(admission.status(), IntentBatchAdmissionStatus::ReviewRequired);
        assert_eq!(admission.batch_id(), BATCH_ID);
    }

    #[test]
    fn stale_missing_evidence_bad_confidence_and_receipt_leaks_fail() {
        // r[verify agent_harness_runtime.turnstone_intent_validation.receipts]
        // r[verify agent_harness_runtime.turnstone_intent_validation.verification]
        let stale = validate_intent_verdict(&IntentVerdictDraft {
            observed_epoch: STALE_EPOCH,
            ..verdict_draft(CALL_READ, IntentRiskLevel::Low, IntentRecommendation::Allow)
        })
        .expect_err("stale verdicts must fail");
        let missing_evidence = validate_intent_verdict(&IntentVerdictDraft {
            evidence_hashes: &[],
            ..verdict_draft(CALL_READ, IntentRiskLevel::Low, IntentRecommendation::Allow)
        })
        .expect_err("missing evidence must fail");
        let bad_confidence = validate_intent_verdict(&IntentVerdictDraft {
            confidence: CONFIDENCE_INVALID,
            ..verdict_draft(CALL_READ, IntentRiskLevel::Low, IntentRecommendation::Allow)
        })
        .expect_err("bad confidence must fail");
        let receipt = validate_intent_receipt(&IntentReceiptDraft {
            batch_id: BATCH_ID,
            status: "recorded",
            verdict_count: SINGLE_VERDICT_COUNT,
            policy_hash: HASH_A,
            evidence_hash: HASH_B,
            redaction_marker: "redacted-safe-labels",
            recommendation: IntentRecommendation::RequireReview,
            issue_code: Some("review-required"),
        })
        .unwrap();
        let leaking = validate_intent_receipt(&IntentReceiptDraft {
            issue_code: Some("raw command arguments leaked"),
            ..IntentReceiptDraft {
                batch_id: BATCH_ID,
                status: "failed",
                verdict_count: SINGLE_VERDICT_COUNT,
                policy_hash: HASH_A,
                evidence_hash: HASH_B,
                redaction_marker: "redacted-safe-labels",
                recommendation: IntentRecommendation::Deny,
                issue_code: Some("review-required"),
            }
        })
        .expect_err("receipt leak must fail");

        assert_eq!(stale.code(), IntentDiagnosticCode::StaleVerdict);
        assert_eq!(missing_evidence.code(), IntentDiagnosticCode::MissingEvidence);
        assert_eq!(bad_confidence.code(), IntentDiagnosticCode::InvalidConfidence);
        assert_eq!(receipt.verdict_count(), SINGLE_VERDICT_COUNT);
        assert_eq!(leaking.code(), IntentDiagnosticCode::UnsafeMetadata);
    }
}
