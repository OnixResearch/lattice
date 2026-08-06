mod response;

pub use response::Response;

/// Approval posture attached to a current-shape tool request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApprovalPosture {
    /// Host policy pre-approved the call.
    PreApproved,
    /// Host policy requires approval before executor effects.
    RequiresApproval,
    /// Host policy denied approval before executor effects.
    Denied,
}

impl ApprovalPosture {
    /// Parses a stable approval posture label.
    ///
    /// # Errors
    ///
    /// Returns [`super::ContractError`] when the label is missing or unsupported.
    pub fn parse(value: &str) -> Result<Self, super::ContractError> {
        let normalized = super::required_text_value(super::APPROVAL_FIELD, value)?;
        match normalized.as_str() {
            super::APPROVAL_PRE_APPROVED => Ok(Self::PreApproved),
            super::APPROVAL_REQUIRES_APPROVAL => Ok(Self::RequiresApproval),
            super::APPROVAL_DENIED => Ok(Self::Denied),
            other => Err(super::ContractError::UnsupportedApprovalPosture {
                posture: super::Text::from(other),
            }),
        }
    }

    /// Returns the stable approval posture label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreApproved => super::APPROVAL_PRE_APPROVED,
            Self::RequiresApproval => super::APPROVAL_REQUIRES_APPROVAL,
            Self::Denied => super::APPROVAL_DENIED,
        }
    }
}

/// Redaction class for runtime-visible bounded tool results.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputRedactionClass {
    /// Visible text is safe model-visible content.
    Safe,
    /// Visible text was redacted by a host policy.
    Redacted,
    /// Output was adjacent to sensitive data and only safe metadata remains.
    SecretAdjacent,
}

impl OutputRedactionClass {
    /// Returns the stable redaction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Safe => super::REDACTION_SAFE,
            Self::Redacted => super::REDACTION_REDACTED,
            Self::SecretAdjacent => super::REDACTION_SECRET_ADJACENT,
        }
    }
}

/// Borrowed current-shape tool request before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestDraft<'a> {
    /// Optional tool name, required by validation.
    pub name: Option<&'a str>,
    /// Prepared current-schema arguments, required by validation.
    pub arguments: Option<&'a super::PreparedArguments>,
    /// Optional tool-call correlation identity, required by validation.
    pub correlation_id: Option<&'a str>,
    /// Runtime-visible output bound policy.
    pub output_policy: super::OutputPolicy,
    /// Opaque mutation targets declared by this call.
    pub mutation_targets: &'a [&'a str],
    /// Exact capability labels required before execution.
    pub required_capability_labels: &'a [&'a str],
    /// Approval posture label.
    pub approval_posture: &'a str,
}

/// Request sent from the harness to a tool component.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    name: super::Text,
    schema_identity: super::Text,
    arguments: super::Collection<super::ArgumentField>,
    correlation_id: super::Text,
    output_policy: super::OutputPolicy,
    mutation_targets: super::Collection<super::Text>,
    required_capability_labels: super::Collection<super::Text>,
    approval_posture: ApprovalPosture,
}

impl Request {
    /// Builds a current-shape tool request.
    ///
    /// # Errors
    ///
    /// Returns [`super::ContractError`] when any required current-shape field is
    /// absent or malformed, output policy is invalid, arguments are missing, or
    /// exact capability labels are absent.
    pub fn from_draft(draft: &RequestDraft<'_>) -> Result<Self, super::ContractError> {
        let name = super::required_text(super::NAME_FIELD, draft.name)?;
        let arguments = draft.arguments.ok_or(super::ContractError::MissingField {
            field: super::ARGUMENT_NAME_FIELD.as_str(),
        })?;
        validate_output_policy(draft.output_policy)?;
        let mutation_targets = super::normalized_label_set(super::MUTATION_TARGET_FIELD, draft.mutation_targets)?;
        let required_capability_labels = required_capability_labels(draft.required_capability_labels)?;
        Ok(Self {
            name,
            schema_identity: super::Text::from(arguments.schema_identity()),
            arguments: arguments.fields().to_vec(),
            correlation_id: super::required_text(super::CORRELATION_FIELD, draft.correlation_id)?,
            output_policy: draft.output_policy,
            mutation_targets,
            required_capability_labels,
            approval_posture: ApprovalPosture::parse(draft.approval_posture)?,
        })
    }

    /// Returns the requested tool name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the current parameter schema identity.
    #[must_use]
    pub fn schema_identity(&self) -> &str {
        self.schema_identity.as_str()
    }

    /// Returns current-shape validated arguments.
    #[must_use]
    pub fn arguments(&self) -> &[super::ArgumentField] {
        self.arguments.as_slice()
    }

    /// Returns the safe tool-call correlation identity.
    #[must_use]
    pub fn correlation_id(&self) -> &str {
        self.correlation_id.as_str()
    }

    /// Returns runtime output policy for this call.
    #[must_use]
    pub const fn output_policy(&self) -> super::OutputPolicy {
        self.output_policy
    }

    /// Returns declared opaque mutation targets.
    #[must_use]
    pub fn mutation_targets(&self) -> &[super::Text] {
        self.mutation_targets.as_slice()
    }

    /// Returns exact required capability labels.
    #[must_use]
    pub fn required_capability_labels(&self) -> &[super::Text] {
        self.required_capability_labels.as_slice()
    }

    /// Returns the approval posture selected before dispatch.
    #[must_use]
    pub const fn approval_posture(&self) -> ApprovalPosture {
        self.approval_posture
    }

    /// Returns a stable model-visible argument summary without raw legacy input.
    #[must_use]
    pub fn argument_summary(&self) -> super::Text {
        let mut summary = super::Text::new();
        for (index, field) in self.arguments.iter().enumerate() {
            if index > super::FIRST_COLLECTION_INDEX {
                summary.push_str(super::ARGUMENT_SUMMARY_SEPARATOR);
            }
            summary.push_str(field.name());
            summary.push_str(super::ARGUMENT_SUMMARY_ASSIGNMENT);
            summary.push_str(field.value());
        }
        summary
    }

    /// Returns the stable tool request contract identity.
    #[must_use]
    pub const fn contract_identity() -> &'static str {
        super::TOOL_REQUEST_CONTRACT_IDENTITY
    }
}

fn validate_output_policy(policy: super::OutputPolicy) -> Result<(), super::ContractError> {
    if (super::MIN_VISIBLE_OUTPUT_CHARS..=super::MAX_VISIBLE_OUTPUT_CHARS).contains(&policy.visible_char_budget) {
        return Ok(());
    }
    Err(super::ContractError::InvalidVisibleBudget {
        min_chars: super::MIN_VISIBLE_OUTPUT_CHARS,
        max_chars: super::MAX_VISIBLE_OUTPUT_CHARS,
        actual_chars: policy.visible_char_budget,
    })
}

fn required_capability_labels(labels: &[&str]) -> Result<super::Collection<super::Text>, super::ContractError> {
    if labels.is_empty() {
        return Err(crate::AdapterContractError::EmptyCollection {
            field: super::CAPABILITY_FIELD.as_str(),
        }
        .into());
    }
    super::normalized_label_set(super::CAPABILITY_FIELD, labels)
}
