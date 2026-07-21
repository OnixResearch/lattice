/// Gateway delivery validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryError {
    /// Safe text contract failed.
    Contract(crate::AdapterContractError),
    /// BLAKE3 hash contract failed.
    InvalidHash(crate::artifact::Error),
    /// Raw artifact bytes were embedded in a request.
    RawArtifactBytesEmbedded,
    /// Delivery receipt contained forbidden sensitive material.
    ReceiptLeak {
        /// Leak class.
        leak: DeliveryReceiptLeak,
    },
}

impl core::fmt::Display for DeliveryError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::InvalidHash(error) => write!(formatter, "{error}"),
            Self::RawArtifactBytesEmbedded => {
                formatter.write_str("delivery request must not embed raw artifact bytes")
            }
            Self::ReceiptLeak { leak } => write!(formatter, "delivery receipt leak: {leak:?}"),
        }
    }
}

impl From<crate::AdapterContractError> for DeliveryError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

fn normalize(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<alloc::string::String, DeliveryError> {
    crate::validation::normalized_adapter_field(field, value)
        .map_err(DeliveryError::Contract)
}

fn normalize_optional(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<alloc::string::String>, DeliveryError> {
    value.map(|text| normalize(field, text)).transpose()
}
