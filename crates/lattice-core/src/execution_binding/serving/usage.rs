//! Per-service model-request coverage of one attempt for #13.
//!
//! `summarize` is pure. It refuses crossed, conflicting and intent-less
//! dispatched records, collapses identical duplicates, keeps an intent
//! without a record as `dispatch_unconfirmed` with unknown usage and groups
//! every request under its declared route and service. Known tokens are
//! summed only from provider-reported counters; unknown usage is counted,
//! never zero-filled.

// r[impl lattice.serving_binding.usage]
// r[impl lattice.serving_binding.usage.scenario.dedup]
// r[impl lattice.serving_binding.usage.scenario.partial]

pub const USAGE_SCHEMA: &str = "lattice.model-request-usage.v1";
/// Matches the broker's request ceiling per attempt.
pub const MAX_REQUESTS_PER_ROUTE: usize = 256;

type Admitted = super::super::admission::bound::Admitted;
type Intent = super::record::DispatchIntent;
type Record = super::record::Observation;

/// Model-request coverage of one attempt usage observation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelRequestCoverage {
    /// No retained request evidence: per-request usage is not observed.
    Unknown,
    /// No-model worker: there is no inference usage to report.
    NoModel,
    Served(Box<Summary>),
    /// No serving declaration: the attempt broker measured every request it
    /// relayed, without attributing any to a service or deployment.
    BrokeredUnattributed(Box<super::brokered::Summary>),
}

impl ModelRequestCoverage {
    /// The dispatched model requests this coverage proves exactly, or `None`
    /// when any intent lacks a terminal record, any request ended uncertain,
    /// or nothing was observed. `indices` narrows brokered evidence to one
    /// broker run's request-index range; served evidence has no index and
    /// counts only for the whole attempt.
    // r[impl lattice.serving_binding.brokered_evidence.scenario.charged]
    #[must_use]
    pub fn observed_requests(&self, indices: Option<&std::ops::Range<u64>>) -> Option<u64> {
        match (self, indices) {
            (Self::BrokeredUnattributed(summary), Some(indices)) => summary.observed(indices),
            (Self::BrokeredUnattributed(summary), None) => {
                summary.is_count_complete.then_some(summary.dispatched_requests)
            }
            (Self::Served(summary), None) => {
                let is_count_complete = summary
                    .services
                    .iter()
                    .flat_map(|service| &service.requests)
                    .all(|entry| !matches!(entry.class, EntryClass::DispatchUnconfirmed | EntryClass::Uncertain));
                let mut dispatched = summary.services.iter().map(|service| service.dispatched_requests);
                is_count_complete.then(|| dispatched.try_fold(0_u64, u64::checked_add))?
            }
            (Self::Unknown | Self::NoModel | Self::Served(_), _) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Summary {
    pub schema: String,
    /// Every dispatched request ended with known provider-reported usage.
    pub is_complete: bool,
    pub services: Vec<Tally>,
    /// Reserved envelopes versus known tokens; never an exact ceiling while usage is unknown.
    pub overrun: super::envelope::Overrun,
}

/// One declared served route; shared deployments are referenced, not charged.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tally {
    pub route_name: String,
    pub route_identity: String,
    pub service_id: String,
    pub model_id: String,
    pub configuration_ref: String,
    pub attribution: super::Attribution,
    pub dispatched_requests: u64,
    pub not_dispatched_requests: u64,
    pub unknown_usage_requests: u64,
    pub known_input_tokens: u64,
    pub known_output_tokens: u64,
    pub requests: Vec<Entry>,
}

/// One exact request; its own deployment generation and replica are kept.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub request_id: String,
    pub ordinal: u64,
    pub class: EntryClass,
    pub outcome: String,
    pub deployment_id: Option<String>,
    pub deployment_generation: Option<u64>,
    pub replica: Option<String>,
    pub resource_consumer_ref: Option<String>,
    pub usage: super::record::Usage,
    pub evidence: Option<super::record::Evidence>,
    pub receipt_identity: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryClass {
    NotDispatched,
    Completed,
    ResponseRejected,
    Uncertain,
    /// A dispatch intent exists without an observation; the outcome is unknown.
    DispatchUnconfirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "conflict", rename_all = "snake_case")]
pub enum Conflict {
    Unserved,
    Mismatch {
        mismatch: super::record::Mismatch,
    },
    ConflictingRecord,
    ConflictingIntent,
    DispatchWithoutIntent,
    IntentForRefusal,
    Overflow,
    Bound,
    /// Brokered evidence for a binding that declares served routes.
    Served,
}

/// Summarize retained intents and records of one admitted binding.
pub fn summarize(
    admitted: &Admitted,
    intents: &[Intent],
    records: &[Record],
) -> Result<ModelRequestCoverage, Conflict> {
    let Some(declaration) = &admitted.serving else {
        let is_empty = intents.is_empty() && records.is_empty();
        return if !is_empty {
            Err(Conflict::Unserved)
        } else if matches!(admitted.model_routes, super::super::request::RouteDeclaration::NoModel {}) {
            Ok(ModelRequestCoverage::NoModel)
        } else {
            Ok(ModelRequestCoverage::Unknown)
        };
    };
    let intents = unique(intents, |intent| super::record::check_intent(admitted, intent), Conflict::ConflictingIntent)?;
    let records =
        unique(records, |record| super::record::check_observation(admitted, record), Conflict::ConflictingRecord)?;
    let mut services = Vec::with_capacity(declaration.routes.len());
    for route in &declaration.routes {
        services.push(service(admitted, route, &intents, &records)?);
    }
    let is_complete = services.iter().all(|usage| {
        usage.unknown_usage_requests == 0 && usage.requests.iter().all(|entry| entry.class != EntryClass::Uncertain)
    });
    debug_assert_eq!(services.len(), declaration.routes.len());
    debug_assert!(services.iter().all(|usage| usage.requests.len() <= MAX_REQUESTS_PER_ROUTE));
    let overrun = super::envelope::summarize(declaration, &services)?;
    Ok(ModelRequestCoverage::Served(Box::new(Summary {
        schema: USAGE_SCHEMA.to_owned(),
        is_complete,
        services,
        overrun,
    })))
}

/// Validate each item and collapse identical duplicates keyed by request id.
pub(super) fn unique<T: PartialEq + HasRequest>(
    items: &[T],
    check: impl Fn(&T) -> Result<(), super::record::Mismatch>,
    conflict: Conflict,
) -> Result<std::collections::BTreeMap<&str, &T>, Conflict> {
    for item in items {
        check(item).map_err(|mismatch| Conflict::Mismatch { mismatch })?;
    }
    let mut keyed = items.iter().map(|item| (item.request_id(), item)).collect::<Vec<_>>();
    keyed.sort_by_key(|(request_id, _)| *request_id);
    let is_conflicting = keyed
        .windows(2)
        .any(|pair| matches!(pair, [(left_id, left), (right_id, right)] if left_id == right_id && left != right));
    if is_conflicting {
        return Err(conflict);
    }
    let unique = keyed.into_iter().collect::<std::collections::BTreeMap<_, _>>();
    debug_assert!(unique.len() <= items.len());
    debug_assert!(unique.keys().all(|request_id| !request_id.is_empty()));
    Ok(unique)
}

pub(super) trait HasRequest {
    fn request_id(&self) -> &str;
}

impl HasRequest for Intent {
    fn request_id(&self) -> &str {
        &self.request_id
    }
}

impl HasRequest for Record {
    fn request_id(&self) -> &str {
        &self.request_id
    }
}

fn service(
    admitted: &Admitted,
    route: &super::Route,
    intents: &std::collections::BTreeMap<&str, &Intent>,
    records: &std::collections::BTreeMap<&str, &Record>,
) -> Result<Tally, Conflict> {
    let mut entries = Vec::with_capacity(MAX_REQUESTS_PER_ROUTE.min(records.len().saturating_add(intents.len())));
    for record in records.values().filter(|record| record.route_name == route.name) {
        entries.push(recorded(record, intents.get(record.request_id.as_str()).copied())?);
    }
    for intent in intents.values().filter(|intent| intent.route_name == route.name) {
        if !records.contains_key(intent.request_id.as_str()) {
            entries.push(super::unconfirmed(intent));
        }
    }
    if entries.len() > MAX_REQUESTS_PER_ROUTE {
        return Err(Conflict::Bound);
    }
    entries.sort_by_key(|entry| entry.ordinal);
    let mut usage = Tally {
        route_name: route.name.clone(),
        route_identity: route_identity(admitted, &route.name),
        service_id: route.service_id.clone(),
        model_id: route.model_id.clone(),
        configuration_ref: route.configuration_ref.clone(),
        attribution: admitted
            .serving
            .as_ref()
            .map_or(super::Attribution::NotApportioned {}, |declaration| declaration.attribution.clone()),
        dispatched_requests: 0,
        not_dispatched_requests: 0,
        unknown_usage_requests: 0,
        known_input_tokens: 0,
        known_output_tokens: 0,
        requests: Vec::new(),
    };
    tally(&mut usage, &entries)?;
    usage.requests = entries;
    debug_assert!(usage.requests.iter().all(|entry| !entry.request_id.is_empty()));
    Ok(usage)
}

fn route_identity(admitted: &Admitted, name: &str) -> String {
    super::declared_route_identity(admitted, name).unwrap_or_default()
}

fn recorded(record: &Record, intent: Option<&Intent>) -> Result<Entry, Conflict> {
    debug_assert!(!record.request_id.is_empty(), "checked records carry a request id");
    debug_assert!(intent.is_none_or(|intent| intent.request_id == record.request_id));
    let served = record.served.as_ref();
    match (record.outcome.class.is_dispatched(), intent) {
        (true, None) => return Err(Conflict::DispatchWithoutIntent),
        (false, Some(_)) => return Err(Conflict::IntentForRefusal),
        (true, Some(intent)) => {
            let is_same = served.is_some_and(|served| {
                served.deployment_id == intent.deployment_id
                    && served.deployment_generation == intent.deployment_generation
            }) && record.admission.grant_ref.as_deref() == Some(intent.grant_ref.as_str())
                && intent.ordinal == record.ordinal;
            if !is_same {
                return Err(Conflict::ConflictingIntent);
            }
        }
        (false, None) => {}
    }
    Ok(Entry {
        request_id: record.request_id.clone(),
        ordinal: record.ordinal,
        class: match record.outcome.class {
            super::record::OutcomeClass::NotDispatched => EntryClass::NotDispatched,
            super::record::OutcomeClass::Completed => EntryClass::Completed,
            super::record::OutcomeClass::ResponseRejected => EntryClass::ResponseRejected,
            super::record::OutcomeClass::Uncertain => EntryClass::Uncertain,
        },
        outcome: record.outcome.label.clone(),
        deployment_id: served.map(|served| served.deployment_id.clone()),
        deployment_generation: served.map(|served| served.deployment_generation),
        replica: served.and_then(|served| served.replica.clone()),
        resource_consumer_ref: served.and_then(|served| served.resource_consumer_ref.clone()),
        usage: record.usage,
        evidence: Some(record.evidence),
        receipt_identity: record.receipt_identity.clone(),
    })
}

fn tally(usage: &mut Tally, entries: &[Entry]) -> Result<(), Conflict> {
    for entry in entries {
        if entry.class == EntryClass::NotDispatched {
            increment(&mut usage.not_dispatched_requests, 1)?;
            continue;
        }
        increment(&mut usage.dispatched_requests, 1)?;
        let super::record::Usage {
            input_tokens: super::record::Counter::Known(input),
            output_tokens: super::record::Counter::Known(output),
            origin: super::record::UsageOrigin::ProviderReported,
        } = entry.usage
        else {
            increment(&mut usage.unknown_usage_requests, 1)?;
            continue;
        };
        increment(&mut usage.known_input_tokens, input)?;
        increment(&mut usage.known_output_tokens, output)?;
    }
    debug_assert!(usage.unknown_usage_requests <= usage.dispatched_requests);
    debug_assert!(entries.len() <= MAX_REQUESTS_PER_ROUTE);
    Ok(())
}

fn increment(total: &mut u64, amount: u64) -> Result<(), Conflict> {
    *total = total.checked_add(amount).ok_or(Conflict::Overflow)?;
    Ok(())
}
