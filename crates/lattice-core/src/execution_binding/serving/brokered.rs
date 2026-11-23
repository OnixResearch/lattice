//! Per-request evidence of the attempt broker's route without a serving
//! declaration (#31).
//!
//! The attempt-local model broker is the worker's only network egress. For a
//! binding without a serving declaration it writes one
//! `lattice.brokered-model-request-dispatch.v1` intent, synced before upstream
//! contact, and one `lattice.brokered-model-request-observation.v1` terminal
//! record after the call ends. Both bind the admitted binding, attempt, fence,
//! route and the broker's request index. Neither names a service, deployment,
//! grant or replica: the evidence measures requests without attributing them.
//!
//! `summarize` is pure. It refuses crossed, conflicting and intent-less
//! records, collapses identical duplicates and keeps an intent without a
//! record as `dispatch_unconfirmed`.

// r[impl lattice.serving_binding.brokered_evidence]
// r[impl lattice.serving_binding.brokered_evidence.scenario.unattributed]

mod ledger;

pub const INTENT_SCHEMA: &str = "lattice.brokered-model-request-dispatch.v1";
pub const RECORD_SCHEMA: &str = "lattice.brokered-model-request-observation.v1";
pub const USAGE_SCHEMA: &str = "lattice.brokered-model-request-usage.v1";
const REQUEST_DOMAIN: &str = "lattice brokered model request v1";
/// One attempt's broker runs: a repair run's first step and at most eight
/// repairs, each within the broker's 256-request ceiling.
pub const MAX_REQUESTS: usize = 2_304;

type Admitted = super::super::admission::bound::Admitted;

/// Written once, synced before the upstream endpoint is contacted.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intent {
    pub schema: String,
    pub binding_identity: String,
    pub attempt_id: String,
    pub fence: u64,
    pub route_name: String,
    pub request_id: String,
    /// The broker's request index: unique per attempt, never reused.
    pub request_index: u64,
}

/// Written once, after the admitted upstream call ended.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub schema: String,
    pub binding_identity: String,
    pub attempt_id: String,
    pub fence: u64,
    pub route_name: String,
    pub request_id: String,
    pub request_index: u64,
    /// `completed` or `uncertain`; a brokered record always follows its intent.
    pub outcome: super::record::Outcome,
    pub usage: super::record::Usage,
    pub provider_receipt_identity: Option<String>,
}

/// The binding and route every request of one broker run is recorded under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    binding_identity: String,
    attempt_id: String,
    fence: u64,
    route_name: String,
}

impl Context {
    #[must_use]
    pub fn new(admitted: &Admitted, route_name: &str) -> Self {
        debug_assert!(admitted.serving.is_none(), "a served binding records through the serving gate");
        Self {
            binding_identity: admitted.binding_identity.clone(),
            attempt_id: admitted.attempt.attempt_id.clone(),
            fence: admitted.attempt.fence,
            route_name: route_name.to_owned(),
        }
    }

    /// The dispatch intent of the broker's request `request_index`.
    #[must_use]
    pub fn intent(&self, request_index: u64) -> Intent {
        Intent {
            schema: INTENT_SCHEMA.to_owned(),
            binding_identity: self.binding_identity.clone(),
            attempt_id: self.attempt_id.clone(),
            fence: self.fence,
            route_name: self.route_name.clone(),
            request_id: request_id(&self.binding_identity, request_index),
            request_index,
        }
    }
}

impl Record {
    /// The terminal record of `intent`.
    #[must_use]
    pub fn settled(
        intent: &Intent,
        outcome: super::record::Outcome,
        usage: super::record::Usage,
        provider_receipt_identity: Option<String>,
    ) -> Self {
        debug_assert!(outcome.class.is_dispatched(), "a brokered record follows an upstream call");
        Self {
            schema: RECORD_SCHEMA.to_owned(),
            binding_identity: intent.binding_identity.clone(),
            attempt_id: intent.attempt_id.clone(),
            fence: intent.fence,
            route_name: intent.route_name.clone(),
            request_id: intent.request_id.clone(),
            request_index: intent.request_index,
            outcome,
            usage,
            provider_receipt_identity,
        }
    }
}

/// The canonical identity of the broker's request `request_index` of one binding.
#[must_use]
pub fn request_id(binding_identity: &str, request_index: u64) -> String {
    let mut hasher = blake3::Hasher::new_derive_key(REQUEST_DOMAIN);
    hasher.update(binding_identity.as_bytes());
    hasher.update(&[0]);
    hasher.update(&request_index.to_le_bytes());
    format!("blake3:{}", hasher.finalize().to_hex())
}

/// Brokered requests of one attempt, measured and not attributed to any service.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Summary {
    pub schema: String,
    /// Every intent has a terminal record and none ended uncertain, so
    /// `dispatched_requests` is the exact number of upstream calls.
    pub is_count_complete: bool,
    pub dispatched_requests: u64,
    /// Intents without a terminal record.
    pub unconfirmed_requests: u64,
    pub uncertain_requests: u64,
    /// Dispatched requests without both provider-reported token counters.
    pub unknown_usage_requests: u64,
    pub known_input_tokens: u64,
    pub known_output_tokens: u64,
    pub requests: Vec<Entry>,
}

/// One brokered request in index order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub request_id: String,
    pub request_index: u64,
    pub route_name: String,
    pub class: super::usage::EntryClass,
    pub outcome: String,
    pub usage: super::record::Usage,
    pub provider_receipt_identity: Option<String>,
}

impl Summary {
    /// The dispatched requests inside `indices` when every one of them has a
    /// terminal record that is not uncertain; `None` otherwise.
    #[must_use]
    pub fn observed(&self, indices: &std::ops::Range<u64>) -> Option<u64> {
        let mut count: u64 = 0;
        for entry in self.requests.iter().filter(|entry| indices.contains(&entry.request_index)) {
            if matches!(
                entry.class,
                super::usage::EntryClass::DispatchUnconfirmed | super::usage::EntryClass::Uncertain
            ) {
                return None;
            }
            count = count.checked_add(1)?;
        }
        debug_assert!(count <= self.dispatched_requests, "a range counts at most every dispatched request");
        Some(count)
    }
}

/// Summarize retained brokered evidence of one binding without a serving declaration.
pub fn summarize(
    admitted: &Admitted,
    intents: &[Intent],
    records: &[Record],
) -> Result<super::usage::ModelRequestCoverage, super::usage::Conflict> {
    if admitted.serving.is_some() {
        return Err(super::usage::Conflict::Served);
    }
    let is_empty = intents.is_empty() && records.is_empty();
    if is_empty && matches!(admitted.model_routes, super::super::request::RouteDeclaration::NoModel {}) {
        return Ok(super::usage::ModelRequestCoverage::NoModel);
    }
    let intents = super::record::unique(
        intents,
        |intent| ledger::check(admitted, (&intent.schema, INTENT_SCHEMA), &ledger::owner_of_intent(intent)),
        super::usage::Conflict::ConflictingIntent,
    )?;
    let records = super::record::unique(
        records,
        |record| ledger::check_record(admitted, record),
        super::usage::Conflict::ConflictingRecord,
    )?;
    let mut entries = Vec::with_capacity(intents.len().min(MAX_REQUESTS));
    for record in records.values() {
        let intent = intents.get(record.request_id.as_str()).ok_or(super::usage::Conflict::DispatchWithoutIntent)?;
        if intent.request_index != record.request_index || intent.route_name != record.route_name {
            return Err(super::usage::Conflict::ConflictingIntent);
        }
        entries.push(ledger::recorded(record));
    }
    for intent in intents.values().filter(|intent| !records.contains_key(intent.request_id.as_str())) {
        entries.push(ledger::unconfirmed(intent));
    }
    if entries.len() > MAX_REQUESTS {
        return Err(super::usage::Conflict::Bound);
    }
    entries.sort_by_key(|entry| entry.request_index);
    let summary = ledger::tally(entries)?;
    debug_assert_eq!(usize::try_from(summary.dispatched_requests).ok(), Some(summary.requests.len()));
    Ok(super::usage::ModelRequestCoverage::BrokeredUnattributed(Box::new(summary)))
}
