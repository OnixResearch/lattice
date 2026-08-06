/// Transport-neutral typed host error.
// r[impl agent_harness_runtime.host_control.transport_neutral]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostTypedError {
    code: String,
    message: String,
}

impl HostTypedError {
    /// Validates a typed host error.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when code or message text is invalid.
    pub fn new(code: &str, message: &str) -> Result<Self, HostControlError> {
        Ok(Self {
            code: required_text(ERROR_CODE_FIELD, code)?,
            message: required_text(ERROR_MESSAGE_FIELD, message)?,
        })
    }

    /// Returns the typed error code.
    #[must_use]
    pub fn code(&self) -> &str {
        self.code.as_str()
    }

    /// Returns the typed error message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

/// Transport-neutral unknown-field set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostUnknownFields {
    fields: Vec<String>,
}

impl HostUnknownFields {
    /// Validates unknown transport fields without interpreting them.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when a field label is invalid.
    pub fn validate(fields: &[&str]) -> Result<Self, HostControlError> {
        Ok(Self {
            fields: fields
                .iter()
                .map(|field| required_text(UNKNOWN_FIELD, field))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    /// Returns preserved unknown field labels.
    #[must_use]
    pub fn fields(&self) -> &[String] {
        self.fields.as_slice()
    }
}

/// Transport-neutral host-control envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostControlEnvelope {
    correlation_id: String,
    protocol_version: String,
    command: TopologyCommand,
    accepted_features: Vec<String>,
    unknown_features: Vec<String>,
}

impl HostControlEnvelope {
    /// Validates a transport-neutral envelope.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when the version, correlation id, or feature labels are invalid.
    pub fn new(
        correlation_id: &str,
        protocol_version: &str,
        command: TopologyCommand,
        features: &[&str],
    ) -> Result<Self, HostControlError> {
        let protocol_version = required_text(VERSION_FIELD, protocol_version)?;
        if protocol_version != PROTOCOL_VERSION {
            return Err(HostControlError::UnsupportedVersion);
        }
        let feature_count = features.len();
        let mut accepted_features = Vec::with_capacity(feature_count);
        let mut unknown_features = Vec::with_capacity(feature_count);
        for feature in features {
            let normalized = required_text(FEATURE_FIELD, feature)?;
            if is_supported_feature(&normalized) {
                accepted_features.push(normalized);
            } else {
                unknown_features.push(normalized);
            }
        }
        Ok(Self {
            correlation_id: required_text(CORRELATION_FIELD, correlation_id)?,
            protocol_version,
            command,
            accepted_features,
            unknown_features,
        })
    }

    /// Returns the correlation identity.
    #[must_use]
    pub fn correlation_id(&self) -> &str {
        self.correlation_id.as_str()
    }

    /// Returns accepted feature labels.
    #[must_use]
    pub fn accepted_features(&self) -> &[String] {
        self.accepted_features.as_slice()
    }

    /// Returns unknown feature labels preserved for host handling.
    #[must_use]
    pub fn unknown_features(&self) -> &[String] {
        self.unknown_features.as_slice()
    }

    /// Returns the validated command.
    #[must_use]
    pub const fn command(&self) -> &TopologyCommand {
        &self.command
    }
}

fn validate_selector(
    snapshot: &HostTopologySnapshot,
    operation: HostOperation,
    selector: Option<&HostSelector>,
) -> Result<Option<HostSelector>, HostControlError> {
    if matches!(operation, HostOperation::ApplyLayout) {
        return Ok(selector.cloned());
    }
    let selector = selector.ok_or(HostControlError::MissingSelector)?;
    if selector.host_id != snapshot.host_id {
        return Err(HostControlError::CrossHostSelector);
    }
    if !snapshot.contains_selector(selector) {
        return Err(HostControlError::UnknownSelector);
    }
    if selector.lifetime == SelectorLifetime::Live && selector.revision != snapshot.revision {
        return Err(HostControlError::StaleSelector);
    }
    Ok(Some(selector.clone()))
}

fn validate_line_budget(value: u32) -> Result<(), HostControlError> {
    if (MIN_HOST_OUTPUT_LINES..=MAX_HOST_OUTPUT_LINES).contains(&value) {
        Ok(())
    } else {
        Err(HostControlError::UnboundedRead)
    }
}

fn parse_value<T>(
    field_name: crate::validation::FieldName,
    value: &str,
    parser: impl FnOnce(&str) -> Option<T>,
) -> Result<T, HostControlError> {
    let normalized = required_text(field_name, value)?;
    parser(normalized.as_str()).ok_or(HostControlError::UnsupportedValue {
        field: field_name.as_str(),
        value: normalized,
    })
}

fn required_text(
    field_name: crate::validation::FieldName,
    value: &str,
) -> Result<String, HostControlError> {
    crate::validation::normalized_adapter_field(field_name, value).map_err(HostControlError::from)
}

fn optional_text(
    field_name: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<String>, HostControlError> {
    value
        .map(|text| required_text(field_name, text))
        .transpose()
}

fn is_unsafe_host_reference(value: &str) -> bool {
    value.starts_with(ABSOLUTE_PATH_PREFIX)
        || value.starts_with(HOME_PATH_PREFIX)
        || value.contains(URI_SCHEME_MARKER)
        || value.contains(WINDOWS_SEPARATOR)
}

fn optional_reference(value: Option<&str>) -> Result<Option<String>, HostControlError> {
    match value {
        Some(reference) => {
            let normalized = required_text(REFERENCE_FIELD, reference)?;
            if is_unsafe_host_reference(&normalized) {
                return Err(HostControlError::UnsafeOutput);
            }
            Ok(Some(normalized))
        }
        None => Ok(None),
    }
}

fn collect_labels(labels: &[&str]) -> Result<Vec<String>, HostControlError> {
    labels
        .iter()
        .map(|label| required_text(LABEL_FIELD, label))
        .collect()
}

fn count_lines(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    text.lines().count()
}

fn visible_lines(text: &str, line_budget: usize) -> String {
    text.lines()
        .take(line_budget)
        .collect::<Vec<_>>()
        .join("\n")
}

fn hash_parts(parts: &[&str]) -> String {
    let mut text = String::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            text.push_str(HASH_SEPARATOR);
        }
        text.push_str(part);
    }
    format!("{}", blake3::hash(text.as_bytes()).to_hex())
}

fn usize_to_u64(value: usize) -> Result<u64, HostControlError> {
    u64::try_from(value).map_err(|_error| HostControlError::UnboundedRead)
}

fn is_supported_feature(feature: &str) -> bool {
    matches!(
        feature,
        FEATURE_LAYOUT | FEATURE_WAITS | FEATURE_BOUNDED_READS
    )
}
