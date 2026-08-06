use super::*;

#[test]
fn duplicate_same_precedence_project_definitions_fail_closed() {
    let first = project_manifest();
    let second = Manifest::validate(&Draft {
        manifest_identity: Some(SECOND_PROJECT_ID),
        ..project_draft()
    })
    .expect("second project manifest should validate");
    let discovery = discover(&[second, first]);

    assert_eq!(discovery.selected().len(), EMPTY_SELECTION_COUNT);
    assert_eq!(discovery.diagnostics().len(), DUPLICATE_DIAGNOSTIC_COUNT);
    assert_eq!(discovery.diagnostics()[0].issue(), DiscoveryIssue::DuplicateDefinitionName);
    assert_eq!(discovery.diagnostics()[0].name(), NAME);
    assert_eq!(discovery.diagnostics()[0].source_scope(), SourceScope::Project);
}

#[test]
fn unadmitted_project_definition_has_no_influence() {
    let manifest = project_manifest();
    let activation = activate(ActivationRequest {
        manifest: &manifest,
        admission: no_admission(),
        policy: full_policy(),
    });

    assert_eq!(activation.status(), ActivationStatus::Denied);
    assert_eq!(activation.admission_status(), AdmissionStatus::Denied);
    assert_eq!(activation.issues(), &[ActivationIssue::ProjectAdmissionRequired]);
    assert!(activation.prompt_fragments().is_empty());
    assert_eq!(activation.selected_model_role(), None);
    assert!(activation.effective_capabilities().is_empty());
    assert!(activation.effective_tools().is_empty());
    assert!(activation.effective_memory_scopes().is_empty());
    assert!(activation.effective_context_reference_scopes().is_empty());
    assert!(activation.effective_workflow_abilities().is_empty());
    assert!(activation.denied_escalations().is_empty());
}

#[test]
fn escalation_requests_are_omitted_and_recorded() {
    let manifest = escalation_manifest();
    let activation = activate(ActivationRequest {
        manifest: &manifest,
        admission: no_admission(),
        policy: full_policy(),
    });

    assert_eq!(activation.status(), ActivationStatus::Active);
    assert_eq!(activation.issues(), &[ActivationIssue::CapabilityEscalationDenied]);
    assert_eq!(activation.selected_model_role(), None);
    assert_eq!(activation.effective_capabilities(), &[Text::from(CAPABILITY)]);
    assert_eq!(activation.effective_tools(), &[Text::from(TOOLSET)]);
    assert_eq!(activation.effective_memory_scopes(), &[Text::from(MEMORY_SCOPE)]);
    assert_eq!(activation.effective_context_reference_scopes(), &[Text::from(CONTEXT_SCOPE)]);
    assert_escalations(activation.denied_escalations());
}

#[test]
fn malformed_manifest_fields_fail_validation() {
    let missing_name = Manifest::validate(&Draft {
        name: None,
        ..user_draft()
    })
    .expect_err("missing name should fail");
    assert_eq!(missing_name, Error::MissingField {
        field: NAME_FIELD.as_str()
    });

    let bad_scope = Manifest::validate(&Draft {
        source_scope: Some(UNSUPPORTED_SCOPE),
        ..user_draft()
    })
    .expect_err("unsupported source scope should fail");
    assert_eq!(bad_scope, Error::UnsupportedValue {
        field: SOURCE_FIELD.as_str(),
        value: Text::from(UNSUPPORTED_SCOPE),
    });
}

#[test]
fn bounded_manifest_collections_and_prompt_budget_fail_closed() {
    let too_many_items = MAX_ITEMS.saturating_add(1);
    let too_many_capabilities = alloc::vec![CAPABILITY; too_many_items];
    let too_many = Manifest::validate(&Draft {
        capability_ceilings: too_many_capabilities.as_slice(),
        ..user_draft()
    })
    .expect_err("too many capability labels should fail");
    assert_eq!(too_many, Error::TooManyItems {
        field: CAPABILITY_FIELD.as_str(),
        max_items: MAX_ITEMS,
        actual_items: too_many_items,
    });

    let over_budget_chars = MAX_PROMPT_CHARS.saturating_add(1);
    let long_prompt = alloc::string::String::from("x").repeat(over_budget_chars);
    let prompt_fragments = [long_prompt.as_str()];
    let prompt_too_long = Manifest::validate(&Draft {
        prompt_fragments: &prompt_fragments,
        ..user_draft()
    })
    .expect_err("prompt budget overflow should fail");
    assert_eq!(prompt_too_long, Error::PromptTooLong {
        max_chars: MAX_PROMPT_CHARS,
        actual_chars: over_budget_chars,
    });
}

#[test]
fn receipt_leak_fixtures_are_rejected() {
    assert_eq!(validate_text(RAW_PROMPT_RECEIPT), Err(ReceiptLeak::RawPromptBody));
    assert_eq!(validate_text(FULL_PATH_RECEIPT), Err(ReceiptLeak::FullPath));
    assert_eq!(validate_text(CREDENTIAL_RECEIPT), Err(ReceiptLeak::Credential));
    assert_eq!(validate_text(PROVIDER_RECEIPT), Err(ReceiptLeak::ProviderPayload));
    assert_eq!(validate_text(MEMORY_RECEIPT), Err(ReceiptLeak::HiddenMemoryBody));
}

fn escalation_manifest() -> Manifest {
    Manifest::validate(&Draft {
        model_preference: Some("unauthorized-model"),
        capability_ceilings: &[CAPABILITY, EXTRA_CAPABILITY],
        toolset_limits: &[TOOLSET, EXTRA_TOOLSET],
        memory_scopes: &[MEMORY_SCOPE, EXTRA_MEMORY_SCOPE],
        context_reference_scopes: &[CONTEXT_SCOPE, NETWORK_CONTEXT_SCOPE],
        workflow_abilities: &[WORKFLOW_ABILITY, EXTRA_WORKFLOW_ABILITY],
        ..user_draft()
    })
    .expect("manifest with escalation requests should still parse as data")
}

fn assert_escalations(denied: &[DeniedEscalation]) {
    assert!(denied.iter().any(|item| item.kind() == AbilityKind::ModelRole));
    assert!(denied.iter().any(|item| item.kind() == AbilityKind::Tool && item.value() == EXTRA_TOOLSET));
    assert!(
        denied
            .iter()
            .any(|item| { item.kind() == AbilityKind::ContextReference && item.value() == NETWORK_CONTEXT_SCOPE })
    );
    assert!(
        denied
            .iter()
            .any(|item| { item.kind() == AbilityKind::Workflow && item.value() == EXTRA_WORKFLOW_ABILITY })
    );
}
