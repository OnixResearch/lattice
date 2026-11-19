//! Bounded durable decision records and replay without inference.

/// Schema of a decision record.
pub const SCHEMA: &str = "lattice.typed-policy-decision.v1";

/// Reserved and observed deterministic operation units, kept distinct; no charge is recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Usage {
    /// The profile's operation budget: a reservation, not a measurement.
    pub reserved_operation_count: u64,
    /// Tree splits or weight products the runtime counted, from a verified receipt.
    pub observed_inference_count: Option<u64>,
    /// Features the runtime normalized, from a verified receipt.
    pub observed_normalization_count: Option<u64>,
}

/// The eventual whole-workflow outcome of the run that made a decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunOutcome {
    /// Every node completed or was skipped.
    Completed,
    /// At least one node failed or did not finish.
    Failed,
}

/// One durable `lattice.typed-policy-decision.v1` record.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    /// The record schema.
    pub schema: String,
    /// The invocation identity.
    pub invocation: String,
    /// The profile name.
    pub profile_id: String,
    /// The profile revision.
    pub profile_version: u32,
    /// The profile identity.
    pub profile: String,
    /// The bound artifact identity.
    pub artifact: String,
    /// The bound model identity.
    pub model: String,
    /// The bound runtime version.
    pub runtime_version: String,
    /// The projected-input identity; absent when projection was refused.
    pub projected_input: Option<String>,
    /// The candidate-set identity.
    pub candidate_set: String,
    /// The declared candidate labels.
    pub candidates: Vec<String>,
    /// The typed outcome.
    pub outcome: super::validation::Outcome,
    /// The routed declared label, when not terminal.
    pub routed_label: Option<String>,
    /// Who chose the routed label.
    pub routed_by: Option<super::validation::Source>,
    /// Why the fallback or terminal path was taken.
    pub fallback_reason: Option<String>,
    /// The raw receipt, when the profile permits retention.
    pub receipt: Option<String>,
    /// The verified receipt identity.
    pub receipt_identity: Option<String>,
    /// Reserved and observed operation units.
    pub usage: Usage,
    /// The eventual whole-workflow outcome, attached when the run is persisted.
    pub workflow_outcome: Option<RunOutcome>,
}

/// The facts of one finished decision.
#[derive(Debug, Clone, Copy)]
pub struct Facts<'a> {
    /// The admitted profile.
    pub admitted: &'a super::admission::Admitted,
    /// The invocation; absent when projection was refused.
    pub invocation: Option<&'a super::binding::Invocation>,
    /// The validated response.
    pub checked: &'a super::validation::Checked,
    /// The resolved route.
    pub route: &'a super::validation::Route,
}

/// Builds the durable record of one decision.
#[must_use]
// r[impl lattice.typed_policy.record]
pub fn build(facts: Facts<'_>) -> Record {
    let profile = &facts.admitted.profile;
    let (routed_label, routed_by, fallback_reason) = route_fields(facts.route, &facts.checked.outcome);
    let receipt = facts.checked.receipt.as_ref();
    let invocation_identity = facts.invocation.map_or_else(
        || super::identity(SCHEMA, &[facts.admitted.identity.as_str()]),
        |invocation| invocation.identity.clone(),
    );
    debug_assert!(routed_label.is_some() == routed_by.is_some());
    debug_assert!(!profile.labels.is_empty());
    Record {
        schema: SCHEMA.to_owned(),
        invocation: invocation_identity,
        profile_id: profile.id.clone(),
        profile_version: profile.version,
        profile: facts.admitted.identity.clone(),
        artifact: profile.cohort.artifact.clone(),
        model: profile.cohort.model.clone(),
        runtime_version: profile.cohort.runtime_version.clone(),
        projected_input: facts.invocation.map(|invocation| invocation.projected_input.clone()),
        candidate_set: super::binding::candidate_set_identity(profile),
        candidates: profile.labels.clone(),
        outcome: facts.checked.outcome.clone(),
        routed_label,
        routed_by,
        fallback_reason,
        receipt: receipt
            .filter(|_| profile.result.is_receipt_retained)
            .map(animus_frozen_policy::DecisionReceipt::encode),
        receipt_identity: receipt.map(|verified| verified.identity.clone()),
        usage: Usage {
            reserved_operation_count: profile.limits.max_operation_count,
            observed_inference_count: receipt.map(|verified| verified.body.cost.inference_operations),
            observed_normalization_count: receipt.map(|verified| verified.body.cost.normalization_operations),
        },
        workflow_outcome: None,
    }
}

fn route_fields(
    route: &super::validation::Route,
    outcome: &super::validation::Outcome,
) -> (Option<String>, Option<super::validation::Source>, Option<String>) {
    match route {
        super::validation::Route::Branch {
            label,
            source: super::validation::Source::Policy,
        } => (Some(label.clone()), Some(super::validation::Source::Policy), None),
        super::validation::Route::Branch {
            label,
            source: super::validation::Source::Fallback,
        } => (Some(label.clone()), Some(super::validation::Source::Fallback), Some(outcome.code())),
        super::validation::Route::Terminal { reason } => (None, None, Some(reason.clone())),
    }
}

/// Why a persisted decision cannot be replayed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayRefusal {
    /// Another record schema.
    Schema,
    /// The profile changed since the decision; the record is unchanged.
    ProfileChanged,
    /// The bound artifact or model changed since the decision.
    ArtifactChanged,
    /// The decision ended terminally; there is no branch to replay.
    Terminal(String),
    /// The routed label is not a declared branch.
    Undeclared(String),
    /// A retained receipt does not verify or disagrees with the record.
    Receipt,
}

impl std::fmt::Display for ReplayRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Schema => write!(formatter, "typed policy replay refused: record schema"),
            Self::ProfileChanged => write!(formatter, "typed policy replay refused: profile changed"),
            Self::ArtifactChanged => write!(formatter, "typed policy replay refused: artifact changed"),
            Self::Terminal(reason) => write!(formatter, "typed policy replay refused: terminal decision ({reason})"),
            Self::Undeclared(label) => {
                write!(formatter, "typed policy replay refused: label `{label}` is not declared")
            }
            Self::Receipt => write!(formatter, "typed policy replay refused: retained receipt mismatch"),
        }
    }
}

impl std::error::Error for ReplayRefusal {}

/// Replays a persisted decision against the current profile and declared branch labels,
/// returning the routed label. It starts no adapter, model or effect work.
///
/// # Errors
///
/// Returns [`ReplayRefusal`] for incompatible, terminal or inconsistent records.
// r[impl lattice.typed_policy.replay]
pub fn replay(
    record: &Record,
    admitted: &super::admission::Admitted,
    declared: &[String],
) -> Result<String, ReplayRefusal> {
    check_compatible(record, admitted)?;
    let Some(label) = record.routed_label.as_ref() else {
        return Err(ReplayRefusal::Terminal(record.fallback_reason.clone().unwrap_or_else(|| record.outcome.code())));
    };
    if !declared.contains(label) {
        return Err(ReplayRefusal::Undeclared(label.clone()));
    }
    check_receipt(record)?;
    debug_assert!(declared.contains(label));
    Ok(label.clone())
}

/// Checks that a persisted decision still binds to `admitted` and that its retained receipt
/// verifies, starting no adapter, model or effect work. Unlike [`replay`], a terminal decision
/// passes: a resumed run carries the stored record forward without routing it again.
///
/// # Errors
///
/// Returns [`ReplayRefusal`] for an incompatible profile, artifact, model or receipt.
// r[impl lattice.typed_policy.replay]
pub fn revalidate(record: &Record, admitted: &super::admission::Admitted) -> Result<(), ReplayRefusal> {
    check_compatible(record, admitted)?;
    check_receipt(record)
}

fn check_compatible(record: &Record, admitted: &super::admission::Admitted) -> Result<(), ReplayRefusal> {
    if record.schema != SCHEMA {
        return Err(ReplayRefusal::Schema);
    }
    if record.profile != admitted.identity {
        return Err(ReplayRefusal::ProfileChanged);
    }
    let cohort = &admitted.profile.cohort;
    if record.artifact != cohort.artifact || record.model != cohort.model {
        return Err(ReplayRefusal::ArtifactChanged);
    }
    Ok(())
}

fn check_receipt(record: &Record) -> Result<(), ReplayRefusal> {
    let Some(encoded) = record.receipt.as_ref() else {
        return Ok(());
    };
    let verified = animus_frozen_policy::verify_receipt(encoded.as_bytes()).map_err(|_| ReplayRefusal::Receipt)?;
    if record.receipt_identity.as_ref() != Some(&verified.identity) {
        return Err(ReplayRefusal::Receipt);
    }
    debug_assert!(verified.identity.starts_with(animus_frozen_policy::BLAKE3_PREFIX));
    debug_assert!(encoded.len() <= animus_frozen_policy::MAX_RECEIPT_BYTES);
    match &record.outcome {
        super::validation::Outcome::Accepted { label, .. } if verified.body.action.as_ref() != Some(label) => {
            Err(ReplayRefusal::Receipt)
        }
        super::validation::Outcome::Accepted { .. }
        | super::validation::Outcome::Abstained
        | super::validation::Outcome::Unavailable { .. }
        | super::validation::Outcome::TimedOut { .. }
        | super::validation::Outcome::Invalid { .. }
        | super::validation::Outcome::Cancelled => Ok(()),
    }
}

/// One decision made at one node during a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeRecord {
    /// The Condition node.
    pub node: crate::engine::workflow::NodeId,
    /// Its decision record.
    pub record: Record,
}

/// Places each decision on its node's detail and attaches the eventual workflow outcome.
// r[impl lattice.typed_policy.record]
pub fn attach(details: &mut crate::store::RunDetailRecord, decisions: Vec<NodeRecord>) {
    let is_completed = details.nodes.iter().all(|node| {
        matches!(
            node.status,
            crate::store::RunNodeDetailStatus::Completed | crate::store::RunNodeDetailStatus::Skipped
        )
    });
    let outcome = if is_completed {
        RunOutcome::Completed
    } else {
        RunOutcome::Failed
    };
    place(details, decisions, Some(outcome));
}

/// Places each decision on its node's detail of a run suspended at an approval gate; the
/// eventual workflow outcome stays absent until the resumed run finishes.
// r[impl lattice.typed_policy.record]
pub fn attach_suspended(details: &mut crate::store::RunDetailRecord, decisions: Vec<NodeRecord>) {
    place(details, decisions, None);
}

fn place(details: &mut crate::store::RunDetailRecord, decisions: Vec<NodeRecord>, outcome: Option<RunOutcome>) {
    debug_assert!(decisions.len() <= details.nodes.len());
    debug_assert!(decisions.iter().all(|decision| decision.record.workflow_outcome.is_none()));
    for decision in decisions {
        if let Some(node) = details.nodes.iter_mut().find(|node| node.node_id == decision.node) {
            node.policy_decision = Some(Record {
                workflow_outcome: outcome,
                ..decision.record
            });
        }
    }
}
