fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= super::MAX_IDENTIFIER_BYTES
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._-:/".contains(&byte))
}

fn identity(value: &str) -> bool {
    value.strip_prefix("b3:").is_some_and(|hex| {
        hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn cohort(value: &super::SourceCohort) -> bool {
    [
        &value.producer_identity,
        &value.selection_identity,
        &value.obligations_identity,
        &value.profile_identity,
        &value.context_identity,
        &value.source_identity,
        &value.policy_identity,
        &value.dependency_identity,
    ]
    .into_iter()
    .all(|value| identity(value))
}

fn validate_context(request: &super::CompileRequest) -> Result<(), super::Diagnostic> {
    let intent = &request.intent;
    if intent.schema != super::INTENT_SCHEMA {
        return Err(super::Diagnostic::new(super::DiagnosticCode::UnsupportedSchema, "schema"));
    }
    if intent.coverage != super::ContextCoverage::Complete || !intent.unsupported_fields.is_empty() {
        return Err(super::Diagnostic::new(super::DiagnosticCode::IncompleteContext, "coverage"));
    }
    if !cohort(&intent.cohort)
        || !identity(&request.facts.compiler_artifact_identity)
        || !identity(&request.facts.runtime_profile_identity)
    {
        return Err(super::Diagnostic::new(super::DiagnosticCode::InvalidIdentity, "cohort"));
    }
    if intent.cohort != request.facts.expected_cohort {
        return Err(super::Diagnostic::new(super::DiagnosticCode::BindingMismatch, "cohort"));
    }
    debug_assert_eq!(intent.schema, super::INTENT_SCHEMA);
    debug_assert_eq!(intent.cohort, request.facts.expected_cohort);
    Ok(())
}

pub(super) fn normalize(request: &mut super::CompileRequest) -> Result<(), super::Diagnostic> {
    validate_context(request)?;
    let intent = &mut request.intent;
    if intent.obligations.is_empty()
        || intent.obligations.len() > super::MAX_OBLIGATIONS
        || intent.alternatives.len() > super::MAX_ALTERNATIVES
    {
        return Err(super::Diagnostic::new(super::DiagnosticCode::LimitExceeded, "input"));
    }
    if request.facts.available_capabilities.len() > super::MAX_CONTRACTS_PER_OBLIGATION
        || request.facts.available_implementations.len() > super::MAX_ALTERNATIVES
    {
        return Err(super::Diagnostic::new(super::DiagnosticCode::LimitExceeded, "input"));
    }
    if request.facts.available_implementations.iter().any(|value| !identity(value))
        || request.facts.available_capabilities.iter().any(|value| !identifier(value))
    {
        return Err(super::Diagnostic::new(super::DiagnosticCode::LimitExceeded, "input"));
    }
    intent.obligations.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    let mut node_ids = std::collections::BTreeSet::new();
    let mut edge_count = 0_usize;
    for obligation in &mut intent.obligations {
        if !identifier(&obligation.id) {
            return Err(super::Diagnostic::new(super::DiagnosticCode::InvalidIdentity, "obligation"));
        }
        if !node_ids.insert(obligation.id.clone()) {
            return Err(super::Diagnostic::new(super::DiagnosticCode::DuplicateIdentity, &obligation.id));
        }
        validate_obligation(obligation, &request.facts)?;
        edge_count = edge_count
            .checked_add(obligation.prerequisites.len())
            .ok_or_else(|| super::Diagnostic::new(super::DiagnosticCode::LimitExceeded, "dependencies"))?;
        if edge_count > super::MAX_DEPENDENCIES {
            return Err(super::Diagnostic::new(super::DiagnosticCode::LimitExceeded, "dependencies"));
        }
    }
    for obligation in &intent.obligations {
        if obligation.prerequisites.iter().any(|id| !node_ids.contains(id) || id == &obligation.id) {
            return Err(super::Diagnostic::new(super::DiagnosticCode::InvalidDependency, &obligation.id));
        }
    }
    validate_alternatives(intent, &node_ids)?;
    if let Some(selected) = &intent.selected_alternative {
        let alternative = intent
            .alternatives
            .iter()
            .find(|alternative| &alternative.id == selected)
            .ok_or_else(|| super::Diagnostic::new(super::DiagnosticCode::AmbiguousAlternative, "alternatives"))?;
        if !request.facts.available_implementations.contains(&alternative.implementation_identity) {
            return Err(super::Diagnostic::new(super::DiagnosticCode::MissingCapability, "implementation"));
        }
    }
    debug_assert_eq!(node_ids.len(), intent.obligations.len());
    debug_assert!(intent.obligations.windows(2).all(|pair| pair[0].id < pair[1].id));
    Ok(())
}

fn validate_obligation(value: &mut super::Obligation, facts: &super::CompilerFacts) -> Result<(), super::Diagnostic> {
    let fail = |code| super::Diagnostic::new(code, &value.id);
    if [
        &value.obligation_identity,
        &value.guidance_identity,
        &value.remaining_condition_identity,
        &value.action_identity,
        &value.precondition_identity,
    ]
    .into_iter()
    .any(|value| !identity(value))
    {
        return Err(fail(super::DiagnosticCode::InvalidIdentity));
    }
    if !matches!(value.operation_class.as_str(), "cairn_validate" | "named_check" | "review_wait" | "completion_only") {
        return Err(fail(super::DiagnosticCode::UnsupportedOperation));
    }
    if value.required_observations.is_empty() {
        return Err(fail(super::DiagnosticCode::MissingContract));
    }
    if value.required_observations.len() > super::MAX_CONTRACTS_PER_OBLIGATION
        || value.prerequisites.len() > super::MAX_DEPENDENCIES
        || value.required_capabilities.len() > super::MAX_CONTRACTS_PER_OBLIGATION
    {
        return Err(fail(super::DiagnosticCode::LimitExceeded));
    }
    if value.prerequisites.iter().any(|prerequisite| !identifier(prerequisite)) {
        return Err(fail(super::DiagnosticCode::InvalidIdentity));
    }
    if value.required_capabilities.iter().any(|capability| !identifier(capability)) {
        return Err(fail(super::DiagnosticCode::InvalidIdentity));
    }
    if !value.required_capabilities.is_subset(&facts.available_capabilities) {
        return Err(fail(super::DiagnosticCode::MissingCapability));
    }
    value.prerequisites.sort_unstable();
    if value.prerequisites.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(fail(super::DiagnosticCode::DuplicateIdentity));
    }
    value.required_observations.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    for contract in &value.required_observations {
        if !identifier(&contract.id) || !identifier(&contract.schema) || !identity(&contract.predicate_identity) {
            return Err(fail(super::DiagnosticCode::InvalidIdentity));
        }
    }
    if value.required_observations.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(fail(super::DiagnosticCode::DuplicateIdentity));
    }
    debug_assert!(value.prerequisites.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(value.required_observations.windows(2).all(|pair| pair[0].id < pair[1].id));
    Ok(())
}

fn validate_alternatives(
    intent: &mut super::Definition,
    nodes: &std::collections::BTreeSet<String>,
) -> Result<(), super::Diagnostic> {
    debug_assert_eq!(nodes.len(), intent.obligations.len());
    intent.alternatives.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    let fail = || super::Diagnostic::new(super::DiagnosticCode::AmbiguousAlternative, "alternatives");
    if intent.alternatives.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(fail());
    }
    debug_assert!(intent.alternatives.windows(2).all(|pair| pair[0].id < pair[1].id));
    for alternative in &intent.alternatives {
        let has_invalid_binding = !identifier(&alternative.id) || !identity(&alternative.implementation_identity);
        if has_invalid_binding {
            return Err(fail());
        }
        if alternative.obligations.is_empty() || !alternative.obligations.is_subset(nodes) {
            return Err(fail());
        }
    }
    match (&intent.selected_alternative, intent.alternatives.as_slice()) {
        (None, []) => Ok(()),
        (Some(selected), alternatives) if alternatives.iter().any(|choice| &choice.id == selected) => Ok(()),
        _ => Err(fail()),
    }
}
