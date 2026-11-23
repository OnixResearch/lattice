//! The checks and the tally behind `brokered::summarize`: each retained
//! intent and record must belong to the admitted binding, a declared route
//! and its own request index, and the summary counts every request exactly
//! once with checked arithmetic.

type Admitted = super::Admitted;
type Conflict = super::super::usage::Conflict;
type EntryClass = super::super::usage::EntryClass;
type Mismatch = super::super::record::Mismatch;
type OutcomeClass = super::super::record::OutcomeClass;

/// Binding identity, attempt, fence, route, request id and request index.
type Owner<'a> = (&'a str, &'a str, u64, &'a str, &'a str, u64);

pub(super) const fn owner_of_intent(intent: &super::Intent) -> Owner<'_> {
    (
        intent.binding_identity.as_str(),
        intent.attempt_id.as_str(),
        intent.fence,
        intent.route_name.as_str(),
        intent.request_id.as_str(),
        intent.request_index,
    )
}

pub(super) fn check_record(admitted: &Admitted, record: &super::Record) -> Result<(), Mismatch> {
    let owner = (
        record.binding_identity.as_str(),
        record.attempt_id.as_str(),
        record.fence,
        record.route_name.as_str(),
        record.request_id.as_str(),
        record.request_index,
    );
    check(admitted, (&record.schema, super::RECORD_SCHEMA), &owner)?;
    let is_settled = match record.outcome.class {
        OutcomeClass::Completed => true,
        OutcomeClass::Uncertain => record.usage == super::super::record::Usage::UNKNOWN,
        OutcomeClass::NotDispatched | OutcomeClass::ResponseRejected => false,
    };
    require(is_settled, Mismatch::Inconsistent)
}

/// The evidence belongs to this binding, a declared route and its own index.
pub(super) fn check(admitted: &Admitted, schema: (&str, &str), owner: &Owner<'_>) -> Result<(), Mismatch> {
    let (binding_identity, attempt_id, fence, route_name, request, request_index) = *owner;
    require(schema.0 == schema.1, Mismatch::Schema)?;
    super::super::record::check_owner(admitted, (binding_identity, attempt_id, fence))?;
    let super::super::super::request::RouteDeclaration::Routes { routes } = &admitted.model_routes else {
        return Err(Mismatch::UndeclaredRoute);
    };
    require(routes.iter().any(|route| route.name == route_name), Mismatch::UndeclaredRoute)?;
    require(request == super::request_id(binding_identity, request_index), Mismatch::Crossed {
        field: "request_id",
    })
}

impl super::super::record::HasRequest for super::Intent {
    fn request_id(&self) -> &str {
        &self.request_id
    }
}

impl super::super::record::HasRequest for super::Record {
    fn request_id(&self) -> &str {
        &self.request_id
    }
}

fn require(condition: bool, mismatch: Mismatch) -> Result<(), Mismatch> {
    if condition { Ok(()) } else { Err(mismatch) }
}

pub(super) fn recorded(record: &super::Record) -> super::Entry {
    super::Entry {
        request_id: record.request_id.clone(),
        request_index: record.request_index,
        route_name: record.route_name.clone(),
        class: match record.outcome.class {
            OutcomeClass::Completed => EntryClass::Completed,
            OutcomeClass::Uncertain => EntryClass::Uncertain,
            OutcomeClass::NotDispatched => EntryClass::NotDispatched,
            OutcomeClass::ResponseRejected => EntryClass::ResponseRejected,
        },
        outcome: record.outcome.label.clone(),
        usage: record.usage,
        provider_receipt_identity: record.provider_receipt_identity.clone(),
    }
}

pub(super) fn unconfirmed(intent: &super::Intent) -> super::Entry {
    super::Entry {
        request_id: intent.request_id.clone(),
        request_index: intent.request_index,
        route_name: intent.route_name.clone(),
        class: EntryClass::DispatchUnconfirmed,
        outcome: "dispatch-unconfirmed".to_owned(),
        usage: super::super::record::Usage::UNKNOWN,
        provider_receipt_identity: None,
    }
}

pub(super) fn tally(requests: Vec<super::Entry>) -> Result<super::Summary, Conflict> {
    let add = |total: u64, amount: u64| total.checked_add(amount).ok_or(Conflict::Overflow);
    let mut summary = super::Summary {
        schema: super::USAGE_SCHEMA.to_owned(),
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
            EntryClass::DispatchUnconfirmed => summary.unconfirmed_requests = add(summary.unconfirmed_requests, 1)?,
            EntryClass::Uncertain => summary.uncertain_requests = add(summary.uncertain_requests, 1)?,
            EntryClass::NotDispatched | EntryClass::Completed | EntryClass::ResponseRejected => {}
        }
        let super::super::record::Usage {
            input_tokens: super::super::record::Counter::Known(input),
            output_tokens: super::super::record::Counter::Known(output),
            origin: super::super::record::UsageOrigin::ProviderReported,
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
