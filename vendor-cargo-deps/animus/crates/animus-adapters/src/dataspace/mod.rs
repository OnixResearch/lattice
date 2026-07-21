mod assertion;
mod orchestration;
mod scope;
mod subscription;

#[cfg(test)]
mod fake;
#[cfg(test)]
mod tests;

/// Actor class attached to a scoped dataspace assertion.
pub type ActorKind = scope::ActorKind;
/// Bounded redacted assertion projected to an optional dataspace backend.
pub type Assertion = assertion::Record;
/// Borrowed input used to validate a bounded dataspace assertion.
pub type AssertionDraft<'a> = assertion::Draft<'a>;
/// Bounded assertion classes published through an optional dataspace adapter.
pub type AssertionKind = assertion::Kind;
/// Redaction marker proving raw sensitive material was not asserted.
pub type AssertionRedaction = assertion::Redaction;
/// Safe lifecycle status class for a dataspace assertion.
pub type AssertionStatus = assertion::Status;
/// Host orchestration event delivered to an optional dataspace adapter.
pub type OrchestrationEvent = orchestration::Event;
/// Borrowed input used to validate a host orchestration event.
pub type OrchestrationEventDraft<'a> = orchestration::Draft<'a>;
/// Host orchestration event kinds accepted by a dataspace adapter.
pub type OrchestrationEventKind = orchestration::EventKind;
/// Explicit tenant/workspace/session/facet scope for dataspace projections.
pub type Scope = scope::Projection;
/// Borrowed input used to validate a dataspace projection scope.
pub type ScopeDraft<'a> = scope::Draft<'a>;
/// Safe query pattern for an optional dataspace backend.
pub type Subscription = subscription::Query;
/// Borrowed input used to validate a safe dataspace subscription.
pub type SubscriptionDraft<'a> = subscription::Draft<'a>;
/// Typed failure raised while validating a safe dataspace subscription.
pub type SubscriptionIssue = subscription::Issue;

/// Host dataspace adapter contract for bounded assertion projection.
pub trait AdapterService {
    /// Error returned by the concrete dataspace adapter implementation.
    type Error;

    /// Publishes one bounded, redacted assertion to the configured dataspace.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when the host adapter cannot publish the assertion.
    fn publish(&mut self, assertion: &Assertion) -> Result<(), Self::Error>;

    /// Subscribes to a validated safe query pattern and returns matching assertions.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when the host adapter cannot satisfy the subscription.
    fn subscribe(
        &mut self,
        subscription: &Subscription,
    ) -> Result<alloc::vec::Vec<Assertion>, Self::Error>;

    /// Receives one host orchestration event such as cancellation or expiry.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when the host adapter cannot process the event.
    fn handle_event(&mut self, event: &OrchestrationEvent) -> Result<(), Self::Error>;
}

fn parse_value<T>(
    field_name: crate::validation::FieldName,
    value: &str,
    parser: impl FnOnce(&str) -> Option<T>,
) -> Result<T, crate::AdapterContractError> {
    let normalized = crate::validation::normalized_adapter_field(field_name, value)?;
    parser(normalized.as_str()).ok_or_else(|| {
        crate::AdapterContractError::UnsupportedDataspaceValue {
            field: field_name.as_str(),
            value: normalized,
        }
    })
}

fn optional_field(
    field_name: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<alloc::string::String>, crate::AdapterContractError> {
    value
        .map(|text| crate::validation::normalized_adapter_field(field_name, text))
        .transpose()
}
