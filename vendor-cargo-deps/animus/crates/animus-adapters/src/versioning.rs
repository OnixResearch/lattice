const CONTRACT_IDENTITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("contract_identity");

// r[impl agent_harness_runtime.boundary_versions.version_admission]
/// Supported public boundary contract identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundaryContractIdentity {
    /// Public model request DTO contract.
    ModelRequest,
    /// Public model response DTO contract.
    ModelResponse,
    /// Public model stream frame DTO contract.
    ModelStreamFrame,
    /// Public tool request DTO contract.
    ToolRequest,
    /// Public tool response DTO contract.
    ToolResponse,
    /// Public safe receipt DTO contract.
    SafeReceipt,
    /// Public adapter-call outcome envelope contract.
    AdapterCallOutcome,
}

impl BoundaryContractIdentity {
    /// Returns the stable contract identity string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ModelRequest => crate::model::MODEL_REQUEST_CONTRACT_IDENTITY,
            Self::ModelResponse => crate::model::MODEL_RESPONSE_CONTRACT_IDENTITY,
            Self::ModelStreamFrame => crate::model::wire::MODEL_STREAM_FRAME_CONTRACT_IDENTITY,
            Self::ToolRequest => crate::tool::TOOL_REQUEST_CONTRACT_IDENTITY,
            Self::ToolResponse => crate::tool::TOOL_RESPONSE_CONTRACT_IDENTITY,
            Self::SafeReceipt => crate::SAFE_RECEIPT_CONTRACT_IDENTITY,
            Self::AdapterCallOutcome => crate::ADAPTER_CALL_OUTCOME_CONTRACT_IDENTITY,
        }
    }
}

/// Parses a supported public boundary contract identity.
///
/// # Errors
///
/// Returns [`crate::AdapterContractError`] when the identity is empty, malformed,
/// or names an unsupported contract/version.
pub fn parse_boundary_contract_identity(
    identity: &str,
) -> Result<BoundaryContractIdentity, crate::AdapterContractError> {
    let normalized =
        crate::validation::normalized_adapter_field(CONTRACT_IDENTITY_FIELD, identity)?;
    debug_assert!(!normalized.is_empty());
    debug_assert_eq!(normalized.trim(), normalized.as_str());
    match normalized.as_str() {
        crate::model::MODEL_REQUEST_CONTRACT_IDENTITY => Ok(BoundaryContractIdentity::ModelRequest),
        crate::model::MODEL_RESPONSE_CONTRACT_IDENTITY => {
            Ok(BoundaryContractIdentity::ModelResponse)
        }
        crate::model::wire::MODEL_STREAM_FRAME_CONTRACT_IDENTITY => {
            Ok(BoundaryContractIdentity::ModelStreamFrame)
        }
        crate::tool::TOOL_REQUEST_CONTRACT_IDENTITY => Ok(BoundaryContractIdentity::ToolRequest),
        crate::tool::TOOL_RESPONSE_CONTRACT_IDENTITY => Ok(BoundaryContractIdentity::ToolResponse),
        crate::SAFE_RECEIPT_CONTRACT_IDENTITY => Ok(BoundaryContractIdentity::SafeReceipt),
        crate::ADAPTER_CALL_OUTCOME_CONTRACT_IDENTITY => {
            Ok(BoundaryContractIdentity::AdapterCallOutcome)
        }
        _unsupported => Err(crate::AdapterContractError::UnsupportedRouterValue {
            field: CONTRACT_IDENTITY_FIELD.as_str(),
            value: normalized,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNSUPPORTED_VERSION: &str = "animus.model-request.v99";
    const BLANK_IDENTITY: &str = "   ";

    #[test]
    fn parser_accepts_supported_contract_identities() {
        // r[verify agent_harness_runtime.boundary_versions.contract_identities.validation]
        let identities = [
            BoundaryContractIdentity::ModelRequest,
            BoundaryContractIdentity::ModelResponse,
            BoundaryContractIdentity::ModelStreamFrame,
            BoundaryContractIdentity::ToolRequest,
            BoundaryContractIdentity::ToolResponse,
            BoundaryContractIdentity::SafeReceipt,
            BoundaryContractIdentity::AdapterCallOutcome,
        ];

        for identity in identities {
            assert_eq!(
                parse_boundary_contract_identity(identity.as_str()),
                Ok(identity)
            );
        }
    }

    #[test]
    fn parser_rejects_missing_or_unsupported_contract_versions() {
        // r[verify agent_harness_runtime.boundary_versions.version_admission.fail_closed]
        let missing = parse_boundary_contract_identity(BLANK_IDENTITY)
            .expect_err("blank identity should fail");
        let identity_parse_error = parse_boundary_contract_identity(UNSUPPORTED_VERSION)
            .expect_err("unsupported version should fail");

        assert!(matches!(
            missing,
            crate::AdapterContractError::EmptyField { .. }
        ));
        assert!(matches!(
            identity_parse_error,
            crate::AdapterContractError::UnsupportedRouterValue { .. }
        ));
    }
}
