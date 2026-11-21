//! Refusals decided before a request's own execution intent exists.
//!
//! Such a refusal is printed only once `dispatched` is settled under the
//! per-work lock, after the attempt-fence is closed when it retains no
//! execution intent (`execution::Context::settle`).

// r[impl lattice.implementation_attempt.consumer.scenario.closure]

type Reason = super::Reason;

/// A typed refusal decided before this request's own execution intent
/// exists.
#[derive(Debug, Clone, PartialEq)]
pub(in super::super) struct Pending {
    reason: Reason,
    detail: Option<(&'static str, serde_json::Value)>,
}

impl Pending {
    pub(in super::super) const fn new(reason: Reason) -> Self {
        Self { reason, detail: None }
    }

    /// `revision_not_admitted` with the lifecycle's own refusal.
    // r[impl lattice.agent_revisions.seam.scenario.attempt]
    pub(in super::super) fn revision(refusal: &lattice_core::agent_revision::Refusal) -> Self {
        Self {
            reason: Reason::RevisionNotAdmitted,
            detail: Some(("revision_refusal", serde_json::json!(refusal))),
        }
    }

    /// A lineage refusal with Kiln's lineage reason.
    // r[impl lattice.implementation_attempt.lineage.scenario.exhausted]
    pub(in super::super) fn lineage(reason: Reason, refusal: &crate::implementation_attempt::lineage::Refusal) -> Self {
        debug_assert!(matches!(reason, Reason::LineageBlocked | Reason::LineageExhausted));
        Self {
            reason,
            detail: Some(("lineage_refusal", serde_json::json!(refusal))),
        }
    }

    pub(in super::super) const fn reason(&self) -> Reason {
        self.reason
    }

    /// Print the record with its settled `dispatched` and return its
    /// diagnostic.
    pub(in super::super) fn print(
        self,
        envelope: &crate::implementation_attempt::ImplementationAttemptEnvelope,
        dispatched: bool,
    ) -> String {
        let mut printed = super::record_with(envelope, self.reason, dispatched);
        if let (Some((field, value)), Some(fields)) = (self.detail, printed.as_object_mut()) {
            fields.insert(field.to_owned(), value);
        }
        println!("{printed}");
        self.reason.message().to_owned()
    }
}

/// Why a step before this request's intent stopped.
#[derive(Debug, Clone, PartialEq)]
pub(in super::super) enum Stop {
    /// A typed refusal, settled and printed by the caller under the lock.
    Refused(Pending),
    /// An error that decides nothing and prints no refusal.
    Error(String),
}

impl From<String> for Stop {
    fn from(message: String) -> Self {
        Self::Error(message)
    }
}
