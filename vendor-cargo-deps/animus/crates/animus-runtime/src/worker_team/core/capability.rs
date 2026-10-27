pub(crate) fn validate_capabilities(
    parent_policy_required: bool,
    capabilities: &[super::super::WorkerCapability],
) -> Result<Vec<super::super::WorkerCapability>, super::super::DelegationFailure> {
    if !parent_policy_required {
        return Err(super::super::DelegationFailure::new(
            super::super::FailureKind::OverbroadAuthority,
            super::super::AUTHORITY_FIELD,
            super::super::OVERBROAD_REASON,
        ));
    }
    assert!(parent_policy_required, "parent policy gate is required");
    super::validate_non_empty_slice(super::super::CAPABILITY_FIELD, capabilities)?;
    assert!(
        !capabilities.is_empty(),
        "capability list must be non-empty"
    );
    let mut seen = std::collections::BTreeSet::new();
    for capability in capabilities {
        let label = validate_one(capability)?;
        if !seen.insert(label) {
            return Err(super::super::DelegationFailure::new(
                super::super::FailureKind::OverbroadAuthority,
                super::super::CAPABILITY_FIELD,
                "duplicate capability labels are ambiguous",
            ));
        }
    }
    Ok(capabilities.to_vec())
}

fn validate_one(
    capability: &super::super::WorkerCapability,
) -> Result<String, super::super::DelegationFailure> {
    match capability {
        super::super::WorkerCapability::ReadContext => Ok(String::from("read-context")),
        super::super::WorkerCapability::ReportFindings => Ok(String::from("report-findings")),
        super::super::WorkerCapability::UseTool { tool_label } => {
            let label = super::validate_label(super::super::CAPABILITY_FIELD, tool_label.as_str())?;
            reject_wildcard(label.as_str())?;
            Ok(format!("tool:{label}"))
        }
        super::super::WorkerCapability::RequestMutation { mutation_kind } => {
            let label =
                super::validate_label(super::super::CAPABILITY_FIELD, mutation_kind.as_str())?;
            reject_wildcard(label.as_str())?;
            Ok(format!("mutation:{label}"))
        }
        super::super::WorkerCapability::BypassParentPolicy => {
            Err(super::super::DelegationFailure::new(
                super::super::FailureKind::OverbroadAuthority,
                super::super::CAPABILITY_FIELD,
                super::super::OVERBROAD_REASON,
            ))
        }
    }
}

fn reject_wildcard(label: &str) -> Result<(), super::super::DelegationFailure> {
    if matches!(
        label,
        super::super::WILDCARD_CAPABILITY_LABEL | super::super::WILDCARD_CAPABILITY_WORD
    ) {
        return Err(super::super::DelegationFailure::new(
            super::super::FailureKind::OverbroadAuthority,
            super::super::CAPABILITY_FIELD,
            super::super::OVERBROAD_REASON,
        ));
    }
    Ok(())
}
