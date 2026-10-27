#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FieldName {
    value: &'static str,
}

impl FieldName {
    const fn new(value: &'static str) -> Self {
        Self { value }
    }

    const fn as_str(self) -> &'static str {
        self.value
    }
}

const LABEL_FIELD: FieldName = FieldName::new("label");
const SERVICE_KIND_FIELD: FieldName = FieldName::new("service_kind");
const SCOPE_FIELD: FieldName = FieldName::new("scope");
const MODEL_SERVICE: &str = "model";
const TOOL_SERVICE: &str = "tool";
const EVENT_SINK_SERVICE: &str = "event-sink";
const POLICY_GATE_SERVICE: &str = "policy-gate";
const QUEUE_SERVICE: &str = "queue";
const CANCELLATION_SERVICE: &str = "cancellation";
const CONFIRMATION_SERVICE: &str = "confirmation";
const PROVENANCE_RECEIPT_SINK_SERVICE: &str = "provenance-receipt-sink";
/// Compatibility model capability label used by default runtime constructors.
pub const DEFAULT_MODEL_CAPABILITY_LABEL: &str = "default-model";
/// Compatibility tool capability label used by default runtime constructors.
pub const DEFAULT_TOOL_CAPABILITY_LABEL: &str = "default-tool";
const NUL_CHARACTER: char = '\0';
const UNSAFE_SCOPE_MARKERS: &[&str] = &[
    "/home/",
    "RAW_PROMPT:",
    "PROVIDER_PAYLOAD:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "AWS_SECRET_ACCESS_KEY",
    "sk-",
];

/// Stable runtime service kind for an explicitly supplied host capability.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeCapabilityServiceKind {
    /// Model dispatch service.
    Model,
    /// Tool dispatch service.
    Tool,
    /// Runtime event sink service.
    EventSink,
    /// Host policy gate service.
    PolicyGate,
    /// Background queue service.
    Queue,
    /// Cancellation service.
    Cancellation,
    /// Confirmation broker service.
    Confirmation,
    /// Provenance receipt sink service.
    ProvenanceReceiptSink,
}

impl RuntimeCapabilityServiceKind {
    /// Returns the stable service-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Model => MODEL_SERVICE,
            Self::Tool => TOOL_SERVICE,
            Self::EventSink => EVENT_SINK_SERVICE,
            Self::PolicyGate => POLICY_GATE_SERVICE,
            Self::Queue => QUEUE_SERVICE,
            Self::Cancellation => CANCELLATION_SERVICE,
            Self::Confirmation => CONFIRMATION_SERVICE,
            Self::ProvenanceReceiptSink => PROVENANCE_RECEIPT_SINK_SERVICE,
        }
    }

    /// Parses a stable service-kind label.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeCapabilityError`] when `value` is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, RuntimeCapabilityError> {
        validate_required_text(SERVICE_KIND_FIELD, value)?;
        match value.trim() {
            MODEL_SERVICE => Ok(Self::Model),
            TOOL_SERVICE => Ok(Self::Tool),
            EVENT_SINK_SERVICE => Ok(Self::EventSink),
            POLICY_GATE_SERVICE => Ok(Self::PolicyGate),
            QUEUE_SERVICE => Ok(Self::Queue),
            CANCELLATION_SERVICE => Ok(Self::Cancellation),
            CONFIRMATION_SERVICE => Ok(Self::Confirmation),
            PROVENANCE_RECEIPT_SINK_SERVICE => Ok(Self::ProvenanceReceiptSink),
            other => Err(RuntimeCapabilityError::new(
                RuntimeCapabilityErrorKind::UnsupportedServiceKind,
                SERVICE_KIND_FIELD,
                other,
            )),
        }
    }
}

/// Borrowed capability descriptor before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeCapabilityDescriptorDraft<'a> {
    /// Stable host-visible capability label.
    pub label: &'a str,
    /// Runtime service kind label.
    pub service_kind: &'a str,
    /// Optional safe scope label.
    pub scope: Option<&'a str>,
}

impl RuntimeCapabilityDescriptorDraft<'_> {
    /// Builds the compatibility model capability descriptor.
    #[must_use]
    pub const fn default_model() -> Self {
        Self {
            label: DEFAULT_MODEL_CAPABILITY_LABEL,
            service_kind: MODEL_SERVICE,
            scope: None,
        }
    }

    /// Builds the compatibility tool capability descriptor.
    #[must_use]
    pub const fn default_tool() -> Self {
        Self {
            label: DEFAULT_TOOL_CAPABILITY_LABEL,
            service_kind: TOOL_SERVICE,
            scope: None,
        }
    }
}

/// Validated runtime host capability descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeCapabilityDescriptor {
    label: String,
    service_kind: RuntimeCapabilityServiceKind,
    scope: Option<String>,
}

impl RuntimeCapabilityDescriptor {
    /// Validates a borrowed descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeCapabilityError`] when required fields are empty,
    /// unsupported, overlong, or unsafe for safe receipts.
    pub fn from_draft(
        draft: RuntimeCapabilityDescriptorDraft<'_>,
    ) -> Result<Self, RuntimeCapabilityError> {
        validate_required_text(LABEL_FIELD, draft.label)?;
        validate_optional_scope(draft.scope)?;
        Ok(Self {
            label: String::from(draft.label.trim()),
            service_kind: RuntimeCapabilityServiceKind::parse(draft.service_kind)?,
            scope: draft.scope.map(str::trim).map(String::from),
        })
    }

    /// Returns the stable capability label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns the service kind.
    #[must_use]
    pub const fn service_kind(&self) -> RuntimeCapabilityServiceKind {
        self.service_kind
    }

    /// Returns the optional safe scope label.
    #[must_use]
    pub fn scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }
}

/// Host-supplied runtime capability registry.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RuntimeCapabilityRegistry {
    descriptors: Vec<RuntimeCapabilityDescriptor>,
}

impl RuntimeCapabilityRegistry {
    /// Builds an empty registry.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            descriptors: Vec::new(),
        }
    }

    /// Builds the compatibility registry for the traditional model/tool runner.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeCapabilityError`] if the built-in descriptors become invalid.
    pub fn compatibility_model_tool() -> Result<Self, RuntimeCapabilityError> {
        Self::from_drafts(&[
            RuntimeCapabilityDescriptorDraft::default_model(),
            RuntimeCapabilityDescriptorDraft::default_tool(),
        ])
    }

    /// Builds a registry from borrowed descriptors.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeCapabilityError`] when any descriptor is invalid or labels duplicate.
    pub fn from_drafts(
        drafts: &[RuntimeCapabilityDescriptorDraft<'_>],
    ) -> Result<Self, RuntimeCapabilityError> {
        let descriptors = drafts
            .iter()
            .copied()
            .map(RuntimeCapabilityDescriptor::from_draft)
            .collect::<Result<Vec<_>, _>>()?;
        Self::from_descriptors(descriptors)
    }

    /// Builds a registry from validated descriptors.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeCapabilityError`] when labels duplicate.
    pub fn from_descriptors(
        descriptors: Vec<RuntimeCapabilityDescriptor>,
    ) -> Result<Self, RuntimeCapabilityError> {
        let mut labels = std::collections::BTreeSet::new();
        for descriptor in &descriptors {
            if !labels.insert(descriptor.label().to_owned()) {
                return Err(RuntimeCapabilityError::new(
                    RuntimeCapabilityErrorKind::DuplicateLabel,
                    LABEL_FIELD,
                    descriptor.label(),
                ));
            }
        }
        Ok(Self { descriptors })
    }

    /// Returns all descriptors in deterministic registry order.
    #[must_use]
    pub fn descriptors(&self) -> &[RuntimeCapabilityDescriptor] {
        self.descriptors.as_slice()
    }

    /// Resolves a required capability by label and service kind.
    #[must_use]
    pub fn resolve(
        &self,
        label: &str,
        service_kind: RuntimeCapabilityServiceKind,
    ) -> RuntimeCapabilityResolution<'_> {
        self.descriptors
            .iter()
            .find(|descriptor| {
                descriptor.label() == label && descriptor.service_kind() == service_kind
            })
            .map_or(RuntimeCapabilityResolution::Absent, |descriptor| {
                RuntimeCapabilityResolution::Resolved { descriptor }
            })
    }

    /// Resolves a required capability by exact label, service kind, and safe scope.
    #[must_use]
    pub fn resolve_exact(
        &self,
        required: &RuntimeCapabilityDescriptor,
    ) -> RuntimeCapabilityResolution<'_> {
        // r[impl agent_harness_runtime.scoped_capabilities.exact_resolution]
        self.descriptors
            .iter()
            .find(|descriptor| {
                descriptor.label() == required.label()
                    && descriptor.service_kind() == required.service_kind()
                    && descriptor.scope() == required.scope()
            })
            .map_or(RuntimeCapabilityResolution::Absent, |descriptor| {
                RuntimeCapabilityResolution::Resolved { descriptor }
            })
    }

    /// Returns true when a service kind is present under any label.
    #[must_use]
    pub fn contains_service(&self, service_kind: RuntimeCapabilityServiceKind) -> bool {
        self.descriptors
            .iter()
            .any(|descriptor| descriptor.service_kind() == service_kind)
    }

    pub(crate) fn model_tool_presence(is_model_present: bool, is_tool_present: bool) -> Self {
        let mut descriptors = Vec::new();
        if is_model_present {
            descriptors.push(RuntimeCapabilityDescriptor {
                label: String::from(DEFAULT_MODEL_CAPABILITY_LABEL),
                service_kind: RuntimeCapabilityServiceKind::Model,
                scope: None,
            });
        }
        if is_tool_present {
            descriptors.push(RuntimeCapabilityDescriptor {
                label: String::from(DEFAULT_TOOL_CAPABILITY_LABEL),
                service_kind: RuntimeCapabilityServiceKind::Tool,
                scope: None,
            });
        }
        Self { descriptors }
    }
}

/// Capability resolution result before dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeCapabilityResolution<'a> {
    /// A matching descriptor was resolved.
    Resolved {
        /// Resolved capability descriptor.
        descriptor: &'a RuntimeCapabilityDescriptor,
    },
    /// No matching descriptor was provided by the host.
    Absent,
}

/// Typed result class for a runtime capability call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeCapabilityOutcome {
    /// Capability call completed.
    Completed,
    /// Required capability was absent.
    Absent,
    /// Capability call was denied.
    Denied,
    /// Capability service was unavailable.
    Unavailable,
    /// Capability service output was malformed.
    Malformed,
    /// Capability call was cancelled.
    Cancelled,
    /// Capability call timed out.
    TimedOut,
}

impl RuntimeCapabilityOutcome {
    /// Returns true only for completed capability calls.
    #[must_use]
    pub const fn allows_dispatch(self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Converts to the adapter-layer outcome kind used in runtime reports.
    #[must_use]
    pub const fn outcome_kind(self) -> animus_adapters::OutcomeKind {
        match self {
            Self::Completed => animus_adapters::OutcomeKind::Completed,
            Self::Absent => animus_adapters::OutcomeKind::Absent,
            Self::Denied => animus_adapters::OutcomeKind::Denied,
            Self::Unavailable => animus_adapters::OutcomeKind::Unavailable,
            Self::Malformed => animus_adapters::OutcomeKind::MalformedOutput,
            Self::Cancelled => animus_adapters::OutcomeKind::Cancelled,
            Self::TimedOut => animus_adapters::OutcomeKind::TimedOut,
        }
    }
}

/// Capability descriptor validation error kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeCapabilityErrorKind {
    /// A required label was empty.
    EmptyLabel,
    /// A capability label was duplicated.
    DuplicateLabel,
    /// A service kind label was unsupported.
    UnsupportedServiceKind,
    /// Scope text was too long, contained NUL, or carried unsafe evidence markers.
    UnsafeScopeText,
}

/// Capability descriptor validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeCapabilityError {
    kind: RuntimeCapabilityErrorKind,
    field: &'static str,
    value: String,
}

impl RuntimeCapabilityError {
    fn new(kind: RuntimeCapabilityErrorKind, field: FieldName, value: &str) -> Self {
        Self {
            kind,
            field: field.as_str(),
            value: String::from(value),
        }
    }

    /// Returns the typed error kind.
    #[must_use]
    pub const fn kind(&self) -> RuntimeCapabilityErrorKind {
        self.kind
    }

    /// Returns the field that failed validation.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns the safe rejected value.
    #[must_use]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

impl core::fmt::Display for RuntimeCapabilityError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{} invalid: {:?}", self.field, self.kind)
    }
}

impl std::error::Error for RuntimeCapabilityError {}

fn validate_required_text(field: FieldName, value: &str) -> Result<(), RuntimeCapabilityError> {
    assert!(!field.as_str().is_empty(), "field name must not be empty");
    let trimmed = value.trim();
    if trimmed.is_empty() {
        let kind = if field == LABEL_FIELD {
            RuntimeCapabilityErrorKind::EmptyLabel
        } else {
            RuntimeCapabilityErrorKind::UnsupportedServiceKind
        };
        return Err(RuntimeCapabilityError::new(kind, field, value));
    }
    validate_text_length_and_nul(field, trimmed)
}

fn validate_optional_scope(scope: Option<&str>) -> Result<(), RuntimeCapabilityError> {
    assert!(
        !SCOPE_FIELD.as_str().is_empty(),
        "scope field must be named"
    );
    const {
        assert!(
            animus_adapters::MAX_ADAPTER_FIELD_CHARS >= animus_adapters::MIN_ADAPTER_FIELD_CHARS,
            "adapter field bounds must be ordered"
        );
    };
    let Some(scope) = scope else {
        return Ok(());
    };
    let trimmed = scope.trim();
    if trimmed.is_empty() {
        return Err(RuntimeCapabilityError::new(
            RuntimeCapabilityErrorKind::UnsafeScopeText,
            SCOPE_FIELD,
            scope,
        ));
    }
    validate_text_length_and_nul(SCOPE_FIELD, trimmed)?;
    let lower = trimmed.to_ascii_lowercase();
    for marker in UNSAFE_SCOPE_MARKERS {
        if lower.contains(&marker.to_ascii_lowercase()) {
            return Err(RuntimeCapabilityError::new(
                RuntimeCapabilityErrorKind::UnsafeScopeText,
                SCOPE_FIELD,
                marker,
            ));
        }
    }
    Ok(())
}

fn validate_text_length_and_nul(
    field: FieldName,
    value: &str,
) -> Result<(), RuntimeCapabilityError> {
    assert!(!field.as_str().is_empty(), "field name must not be empty");
    const {
        assert!(
            animus_adapters::MAX_ADAPTER_FIELD_CHARS >= animus_adapters::MIN_ADAPTER_FIELD_CHARS,
            "adapter field bounds must be ordered"
        );
    };
    let actual_chars = value.chars().count();
    if actual_chars > animus_adapters::MAX_ADAPTER_FIELD_CHARS {
        return Err(RuntimeCapabilityError::new(
            RuntimeCapabilityErrorKind::UnsafeScopeText,
            field,
            value,
        ));
    }
    if value.chars().any(|character| character == NUL_CHARACTER) {
        return Err(RuntimeCapabilityError::new(
            RuntimeCapabilityErrorKind::UnsafeScopeText,
            field,
            value,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CUSTOM_MODEL_LABEL: &str = "primary-model";
    const EVENT_SCOPE: &str = "runtime-events";
    const UNKNOWN_SERVICE_KIND: &str = "ambient-provider";
    const UNSAFE_SCOPE: &str = "API_KEY=secret";

    #[test]
    fn descriptor_validation_accepts_typed_services() {
        // r[verify agent_harness_runtime.scoped_capabilities.required_identity.validation]
        // r[verify agent_harness_runtime.scoped_capabilities.exact_resolution.fail_closed]
        let registry = RuntimeCapabilityRegistry::from_drafts(&[
            RuntimeCapabilityDescriptorDraft {
                label: CUSTOM_MODEL_LABEL,
                service_kind: MODEL_SERVICE,
                scope: None,
            },
            RuntimeCapabilityDescriptorDraft {
                label: EVENT_SCOPE,
                service_kind: EVENT_SINK_SERVICE,
                scope: Some(EVENT_SCOPE),
            },
        ])
        .expect("registry should validate");

        assert_eq!(registry.descriptors().len(), 2);
        let required = RuntimeCapabilityDescriptor::from_draft(RuntimeCapabilityDescriptorDraft {
            label: EVENT_SCOPE,
            service_kind: EVENT_SINK_SERVICE,
            scope: Some(EVENT_SCOPE),
        })
        .expect("required capability should validate");
        let mismatched_scope =
            RuntimeCapabilityDescriptor::from_draft(RuntimeCapabilityDescriptorDraft {
                label: EVENT_SCOPE,
                service_kind: EVENT_SINK_SERVICE,
                scope: Some(CUSTOM_MODEL_LABEL),
            })
            .expect("mismatched capability should validate");

        assert!(matches!(
            registry.resolve(CUSTOM_MODEL_LABEL, RuntimeCapabilityServiceKind::Model),
            RuntimeCapabilityResolution::Resolved { .. }
        ));
        assert!(matches!(
            registry.resolve_exact(&required),
            RuntimeCapabilityResolution::Resolved { .. }
        ));
        assert_eq!(
            registry.resolve_exact(&mismatched_scope),
            RuntimeCapabilityResolution::Absent
        );
        assert!(registry.contains_service(RuntimeCapabilityServiceKind::EventSink));
    }

    #[test]
    fn descriptor_validation_rejects_duplicate_empty_unsupported_and_unsafe_scope() {
        // r[verify agent_harness_runtime.scoped_capabilities.required_identity.validation]
        let duplicate = RuntimeCapabilityRegistry::from_drafts(&[
            RuntimeCapabilityDescriptorDraft::default_model(),
            RuntimeCapabilityDescriptorDraft::default_model(),
        ])
        .expect_err("duplicate labels should fail");
        let empty = RuntimeCapabilityDescriptor::from_draft(RuntimeCapabilityDescriptorDraft {
            label: "   ",
            service_kind: MODEL_SERVICE,
            scope: None,
        })
        .expect_err("empty labels should fail");
        let service_kind_rejection =
            RuntimeCapabilityDescriptor::from_draft(RuntimeCapabilityDescriptorDraft {
                label: CUSTOM_MODEL_LABEL,
                service_kind: UNKNOWN_SERVICE_KIND,
                scope: None,
            })
            .expect_err("unsupported services should fail");
        let unsafe_scope =
            RuntimeCapabilityDescriptor::from_draft(RuntimeCapabilityDescriptorDraft {
                label: CUSTOM_MODEL_LABEL,
                service_kind: MODEL_SERVICE,
                scope: Some(UNSAFE_SCOPE),
            })
            .expect_err("unsafe scope should fail");

        assert_eq!(duplicate.kind(), RuntimeCapabilityErrorKind::DuplicateLabel);
        assert_eq!(empty.kind(), RuntimeCapabilityErrorKind::EmptyLabel);
        assert_eq!(
            service_kind_rejection.kind(),
            RuntimeCapabilityErrorKind::UnsupportedServiceKind
        );
        assert_eq!(
            unsafe_scope.kind(),
            RuntimeCapabilityErrorKind::UnsafeScopeText
        );
    }

    #[test]
    fn fail_closed_outcomes_block_dispatch_until_completed() {
        let blocking = [
            RuntimeCapabilityOutcome::Absent,
            RuntimeCapabilityOutcome::Denied,
            RuntimeCapabilityOutcome::Unavailable,
            RuntimeCapabilityOutcome::Malformed,
            RuntimeCapabilityOutcome::Cancelled,
            RuntimeCapabilityOutcome::TimedOut,
        ];

        for outcome in blocking {
            assert!(!outcome.allows_dispatch());
            assert_ne!(
                outcome.outcome_kind(),
                animus_adapters::OutcomeKind::Completed
            );
        }
        assert!(RuntimeCapabilityOutcome::Completed.allows_dispatch());
    }
}
