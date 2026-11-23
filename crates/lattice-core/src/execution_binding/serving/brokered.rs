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
    let intents = super::usage::unique(
        intents,
        |intent| check(admitted, (&intent.schema, INTENT_SCHEMA), &owner_of_intent(intent)),
        super::usage::Conflict::ConflictingIntent,
    )?;
    let records = super::usage::unique(
        records,
        |record| check_record(admitted, record),
        super::usage::Conflict::ConflictingRecord,
    )?;
    let mut entries = Vec::with_capacity(intents.len().min(MAX_REQUESTS));
    for record in records.values() {
        let intent = intents.get(record.request_id.as_str()).ok_or(super::usage::Conflict::DispatchWithoutIntent)?;
        if intent.request_index != record.request_index || intent.route_name != record.route_name {
            return Err(super::usage::Conflict::ConflictingIntent);
        }
        entries.push(recorded(record));
    }
    for intent in intents.values().filter(|intent| !records.contains_key(intent.request_id.as_str())) {
        entries.push(unconfirmed(intent));
    }
    if entries.len() > MAX_REQUESTS {
        return Err(super::usage::Conflict::Bound);
    }
    entries.sort_by_key(|entry| entry.request_index);
    let summary = tally(entries)?;
    debug_assert_eq!(usize::try_from(summary.dispatched_requests).ok(), Some(summary.requests.len()));
    Ok(super::usage::ModelRequestCoverage::BrokeredUnattributed(Box::new(summary)))
}

/// Binding identity, attempt, fence, route, request id and request index.
type Owner<'a> = (&'a str, &'a str, u64, &'a str, &'a str, u64);

const fn owner_of_intent(intent: &Intent) -> Owner<'_> {
    (
        intent.binding_identity.as_str(),
        intent.attempt_id.as_str(),
        intent.fence,
        intent.route_name.as_str(),
        intent.request_id.as_str(),
        intent.request_index,
    )
}

fn check_record(admitted: &Admitted, record: &Record) -> Result<(), super::record::Mismatch> {
    let owner = (
        record.binding_identity.as_str(),
        record.attempt_id.as_str(),
        record.fence,
        record.route_name.as_str(),
        record.request_id.as_str(),
        record.request_index,
    );
    check(admitted, (&record.schema, RECORD_SCHEMA), &owner)?;
    let is_settled = match record.outcome.class {
        super::record::OutcomeClass::Completed => true,
        super::record::OutcomeClass::Uncertain => record.usage == super::record::Usage::UNKNOWN,
        super::record::OutcomeClass::NotDispatched | super::record::OutcomeClass::ResponseRejected => false,
    };
    require(is_settled, super::record::Mismatch::Inconsistent)
}

/// The evidence belongs to this binding, a declared route and its own index.
fn check(admitted: &Admitted, schema: (&str, &str), owner: &Owner<'_>) -> Result<(), super::record::Mismatch> {
    let (binding_identity, attempt_id, fence, route_name, request, request_index) = *owner;
    require(schema.0 == schema.1, super::record::Mismatch::Schema)?;
    super::record::check_owner(admitted, (binding_identity, attempt_id, fence))?;
    let super::super::request::RouteDeclaration::Routes { routes } = &admitted.model_routes else {
        return Err(super::record::Mismatch::UndeclaredRoute);
    };
    require(routes.iter().any(|route| route.name == route_name), super::record::Mismatch::UndeclaredRoute)?;
    require(request == request_id(binding_identity, request_index), super::record::Mismatch::Crossed {
        field: "request_id",
    })
}

impl super::usage::HasRequest for Intent {
    fn request_id(&self) -> &str {
        &self.request_id
    }
}

impl super::usage::HasRequest for Record {
    fn request_id(&self) -> &str {
        &self.request_id
    }
}

fn require(condition: bool, mismatch: super::record::Mismatch) -> Result<(), super::record::Mismatch> {
    if condition { Ok(()) } else { Err(mismatch) }
}

fn recorded(record: &Record) -> Entry {
    Entry {
        request_id: record.request_id.clone(),
        request_index: record.request_index,
        route_name: record.route_name.clone(),
        class: match record.outcome.class {
            super::record::OutcomeClass::Completed => super::usage::EntryClass::Completed,
            super::record::OutcomeClass::Uncertain => super::usage::EntryClass::Uncertain,
            super::record::OutcomeClass::NotDispatched => super::usage::EntryClass::NotDispatched,
            super::record::OutcomeClass::ResponseRejected => super::usage::EntryClass::ResponseRejected,
        },
        outcome: record.outcome.label.clone(),
        usage: record.usage,
        provider_receipt_identity: record.provider_receipt_identity.clone(),
    }
}

fn unconfirmed(intent: &Intent) -> Entry {
    Entry {
        request_id: intent.request_id.clone(),
        request_index: intent.request_index,
        route_name: intent.route_name.clone(),
        class: super::usage::EntryClass::DispatchUnconfirmed,
        outcome: "dispatch-unconfirmed".to_owned(),
        usage: super::record::Usage::UNKNOWN,
        provider_receipt_identity: None,
    }
}

fn tally(requests: Vec<Entry>) -> Result<Summary, super::usage::Conflict> {
    let add = |total: u64, amount: u64| total.checked_add(amount).ok_or(super::usage::Conflict::Overflow);
    let mut summary = Summary {
        schema: USAGE_SCHEMA.to_owned(),
        is_count_complete: false,
        dispatched_requests: 0,
        unconfirmed_requests: 0,
        uncertain_requests: 0,
        unknown_usage_requests: 0,
        known_input_tokens: 0,
        known_output_tokens: 0,
        requests: Vec::new(),
    };
    for entry in &requests {
        summary.dispatched_requests = add(summary.dispatched_requests, 1)?;
        match entry.class {
            super::usage::EntryClass::DispatchUnconfirmed => {
                summary.unconfirmed_requests = add(summary.unconfirmed_requests, 1)?;
            }
            super::usage::EntryClass::Uncertain => summary.uncertain_requests = add(summary.uncertain_requests, 1)?,
            super::usage::EntryClass::NotDispatched
            | super::usage::EntryClass::Completed
            | super::usage::EntryClass::ResponseRejected => {}
        }
        let super::record::Usage {
            input_tokens: super::record::Counter::Known(input),
            output_tokens: super::record::Counter::Known(output),
            origin: super::record::UsageOrigin::ProviderReported,
        } = entry.usage
        else {
            summary.unknown_usage_requests = add(summary.unknown_usage_requests, 1)?;
            continue;
        };
        summary.known_input_tokens = add(summary.known_input_tokens, input)?;
        summary.known_output_tokens = add(summary.known_output_tokens, output)?;
    }
    summary.is_count_complete = summary.unconfirmed_requests == 0 && summary.uncertain_requests == 0;
    summary.requests = requests;
    debug_assert!(summary.unknown_usage_requests <= summary.dispatched_requests);
    Ok(summary)
}
