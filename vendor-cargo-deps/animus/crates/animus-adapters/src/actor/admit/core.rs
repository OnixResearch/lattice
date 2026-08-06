/// Evaluates activation after discovery, project admission, and runtime policy.
#[must_use]
pub fn activate(request: super::ActivationRequest<'_>) -> super::Activation {
    assert!(request.policy.model_roles.len() <= super::super::MAX_ITEMS, "model-role policy stays bounded");
    assert!(request.policy.capabilities.len() <= super::super::MAX_ITEMS, "capability policy stays bounded");
    let admission = project_admission_status(request.manifest, request.admission);
    if admission == super::AdmissionStatus::Denied {
        return denied_activation(admission);
    }
    active_activation(request, admission)
}

fn denied_activation(admission_status: super::AdmissionStatus) -> super::Activation {
    let issues = alloc::vec![super::ActivationIssue::ProjectAdmissionRequired];
    super::Activation {
        status: super::ActivationStatus::Denied,
        admission_status,
        issues,
        prompt_fragments: super::super::Collection::new(),
        selected_model_role: None,
        effective_capabilities: super::super::Collection::new(),
        effective_tools: super::super::Collection::new(),
        effective_memory_scopes: super::super::Collection::new(),
        effective_context_reference_scopes: super::super::Collection::new(),
        effective_workflow_abilities: super::super::Collection::new(),
        denied_escalations: super::super::Collection::new(),
    }
}

fn active_activation(
    request: super::ActivationRequest<'_>,
    admission_status: super::AdmissionStatus,
) -> super::Activation {
    assert!(
        request.manifest.prompt_fragments().len() <= super::super::MAX_ITEMS,
        "activation prompt fragments are bounded"
    );
    assert_eq!(
        admission_status,
        project_admission_status(request.manifest, request.admission),
        "activation admission status matches the request"
    );
    let mut denied = super::super::Collection::with_capacity(super::super::MAX_ITEMS);
    let selected_model_role = select_model_role(request.manifest, request.policy.model_roles, &mut denied);
    let sets = effective_sets(request, &mut denied);
    build_active(ActiveInput {
        manifest: request.manifest,
        admission_status,
        selected_model_role,
        sets,
        denied_escalations: denied,
    })
}

struct EffectiveSets {
    capabilities: super::super::Collection<super::super::Text>,
    tools: super::super::Collection<super::super::Text>,
    memory_scopes: super::super::Collection<super::super::Text>,
    context_reference_scopes: super::super::Collection<super::super::Text>,
    workflow_abilities: super::super::Collection<super::super::Text>,
}

struct ActiveInput<'a> {
    manifest: &'a super::super::Manifest,
    admission_status: super::AdmissionStatus,
    selected_model_role: Option<super::super::Text>,
    sets: EffectiveSets,
    denied_escalations: super::super::Collection<super::DeniedEscalation>,
}

fn effective_sets(
    request: super::ActivationRequest<'_>,
    denied: &mut super::super::Collection<super::DeniedEscalation>,
) -> EffectiveSets {
    assert!(
        request.manifest.capability_ceilings().len() <= super::super::MAX_ITEMS,
        "capability requests are bounded"
    );
    assert!(
        request.manifest.workflow_abilities().len() <= super::super::MAX_ITEMS,
        "workflow requests are bounded"
    );
    EffectiveSets {
        capabilities: filter_allowed(
            super::AbilityKind::Capability,
            request.manifest.capability_ceilings(),
            request.policy.capabilities,
            denied,
        ),
        tools: filter_allowed(
            super::AbilityKind::Tool,
            request.manifest.toolset_limits(),
            request.policy.tools,
            denied,
        ),
        memory_scopes: filter_allowed(
            super::AbilityKind::Memory,
            request.manifest.memory_scopes(),
            request.policy.memory_scopes,
            denied,
        ),
        context_reference_scopes: filter_allowed(
            super::AbilityKind::ContextReference,
            request.manifest.context_reference_scopes(),
            request.policy.context_reference_scopes,
            denied,
        ),
        workflow_abilities: filter_allowed(
            super::AbilityKind::Workflow,
            request.manifest.workflow_abilities(),
            request.policy.workflow_abilities,
            denied,
        ),
    }
}

fn build_active(input: ActiveInput<'_>) -> super::Activation {
    let mut issues = super::super::Collection::with_capacity(1);
    if !input.denied_escalations.is_empty() {
        issues.push(super::ActivationIssue::CapabilityEscalationDenied);
    }
    super::Activation {
        status: super::ActivationStatus::Active,
        admission_status: input.admission_status,
        issues,
        prompt_fragments: input.manifest.prompt_fragments.clone(),
        selected_model_role: input.selected_model_role,
        effective_capabilities: input.sets.capabilities,
        effective_tools: input.sets.tools,
        effective_memory_scopes: input.sets.memory_scopes,
        effective_context_reference_scopes: input.sets.context_reference_scopes,
        effective_workflow_abilities: input.sets.workflow_abilities,
        denied_escalations: input.denied_escalations,
    }
}

fn project_admission_status(
    manifest: &super::super::Manifest,
    admission: super::Admission<'_>,
) -> super::AdmissionStatus {
    if manifest.source_scope() != super::super::SourceScope::Project {
        return super::AdmissionStatus::NotRequired;
    }
    if super::super::contains_label(admission.admitted_manifest_identities, manifest.manifest_identity()) {
        return super::AdmissionStatus::Admitted;
    }
    super::AdmissionStatus::Denied
}

fn select_model_role(
    manifest: &super::super::Manifest,
    model_roles: &[&str],
    denied_escalations: &mut super::super::Collection<super::DeniedEscalation>,
) -> Option<super::super::Text> {
    if super::super::contains_label(model_roles, manifest.model_preference()) {
        return Some(super::super::Text::from(manifest.model_preference()));
    }
    denied_escalations.push(denied_escalation(super::AbilityKind::ModelRole, manifest.model_preference()));
    None
}

fn filter_allowed(
    kind: super::AbilityKind,
    requested: &[super::super::Text],
    allowed: &[&str],
    denied_escalations: &mut super::super::Collection<super::DeniedEscalation>,
) -> super::super::Collection<super::super::Text> {
    assert!(requested.len() <= super::super::MAX_ITEMS, "requested labels are bounded");
    assert!(allowed.len() <= super::super::MAX_ITEMS, "allowed labels are bounded");
    let mut effective = super::super::Collection::with_capacity(requested.len());
    for value in requested {
        if super::super::contains_label(allowed, value.as_str()) {
            effective.push(value.clone());
        } else {
            denied_escalations.push(denied_escalation(kind, value));
        }
    }
    effective
}

fn denied_escalation(kind: super::AbilityKind, value: &str) -> super::DeniedEscalation {
    super::DeniedEscalation {
        kind,
        value: super::super::Text::from(value),
    }
}
