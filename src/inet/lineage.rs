//! Explicit fresh recomputation and consumer supersession decisions.
//!
//! The durable operation-run owner already refuses a second dispatch of one
//! invocation identity. A recomputation is therefore a new invocation whose
//! arguments name a new attempt and its reconciled predecessor; nothing here
//! resumes a heap, pointer graph, allocator or process snapshot. Admitting a
//! recomputation never asserts that the predecessor did not run.

/// The durable facts about the predecessor attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Predecessor<'a> {
    pub invocation: &'a str,
    pub attempt: &'a str,
    pub state: crate::operation::RunState,
    /// An operator or owner reconciled an unsettled `Started` attempt.
    pub is_reconciled: bool,
}

/// The attempt budget carried across fresh recomputations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    pub maximum_attempts: u32,
    pub consumed_attempts: u32,
}

/// An admitted fresh recomputation with retained lineage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recomputation {
    pub predecessor_invocation: String,
    pub predecessor_attempt: String,
    pub attempt: String,
    pub consumed_attempts: u32,
    pub remaining_attempts: u32,
}

/// Admit one explicit fresh recomputation after reconciliation.
pub fn admit_recomputation(
    predecessor: &Predecessor<'_>,
    attempt: &str,
    budget: Budget,
) -> Result<Recomputation, super::Error> {
    let is_settled = predecessor.state != crate::operation::RunState::Started || predecessor.is_reconciled;
    let is_fresh = !attempt.is_empty() && attempt != predecessor.attempt && !predecessor.invocation.is_empty();
    let consumed_attempts = budget.consumed_attempts.checked_add(1);
    let Some(consumed_attempts) = consumed_attempts.filter(|consumed| *consumed <= budget.maximum_attempts) else {
        return Err(super::Error::RecomputationRefused);
    };
    if !is_settled || !is_fresh {
        return Err(super::Error::RecomputationRefused);
    }
    debug_assert!(consumed_attempts <= budget.maximum_attempts);
    Ok(Recomputation {
        predecessor_invocation: predecessor.invocation.to_string(),
        predecessor_attempt: predecessor.attempt.to_string(),
        attempt: attempt.to_string(),
        consumed_attempts,
        remaining_attempts: budget.maximum_attempts.saturating_sub(consumed_attempts),
    })
}

/// Consumer-side admission of a published result: only an authoritative
/// complete result whose consumer binding is still current is usable.
/// The consumer remains the owner of that binding and of every later check.
pub fn admit_for_consumer(document: &serde_json::Value, current_binding: &str) -> Result<i64, super::Error> {
    let is_complete = document["evaluation"] == "complete" && document["authoritative"] == true;
    if !is_complete {
        return Err(super::Error::RecomputationRefused);
    }
    if document["binding"]["consumer_binding"].as_str() != Some(current_binding) {
        return Err(super::Error::SupersededConsumerBinding);
    }
    Ok(crate::pure_inet_profile::canonical_i64_from_json(&document["result"])?)
}
