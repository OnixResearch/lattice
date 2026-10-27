use super::*;

#[test]
fn valid_user_manifest_is_typed_data_without_authority_grants() {
    let manifest = user_manifest();

    assert_eq!(manifest.name(), NAME);
    assert_eq!(manifest.description(), DESCRIPTION);
    assert_eq!(manifest.model_preference(), MODEL_ROLE);
    assert_eq!(manifest.prompt_fragments().len(), 2);
    assert_eq!(manifest.capability_ceilings(), &[Text::from(CAPABILITY)]);
    assert_eq!(manifest.toolset_limits(), &[Text::from(TOOLSET)]);
    assert_eq!(manifest.memory_scopes(), &[Text::from(MEMORY_SCOPE)]);
    assert_eq!(
        manifest.context_reference_scopes(),
        &[Text::from(CONTEXT_SCOPE)]
    );
    assert_eq!(
        manifest.workflow_abilities(),
        &[Text::from(WORKFLOW_ABILITY)]
    );
    assert_eq!(manifest.source_scope(), SourceScope::User);
    assert_eq!(manifest.trust_status(), TrustStatus::Trusted);
    assert_eq!(manifest.manifest_identity(), USER_ID);
    assert_eq!(
        manifest.authority_boundary(),
        AuthorityBoundary::CeilingOnly
    );
}

#[test]
fn deterministic_discovery_prefers_host_then_user_over_project() {
    let project = project_manifest();
    let user = user_manifest();
    let host = Manifest::validate(&host_draft()).expect("host manifest should validate");
    let discovery = discover(&[project, user, host]);

    assert!(discovery.diagnostics().is_empty());
    assert_eq!(discovery.selected().len(), 1);
    assert_eq!(
        discovery.selected()[0].source_scope(),
        SourceScope::ExplicitHost
    );
    assert_eq!(discovery.selected()[0].manifest_identity(), HOST_ID);
}

#[test]
fn admitted_project_activation_applies_only_policy_allowed_ceilings() {
    let manifest = project_manifest();
    let activation = activate(ActivationRequest {
        manifest: &manifest,
        admission: admitted_project(),
        policy: full_policy(),
    });

    assert_eq!(activation.status(), ActivationStatus::Active);
    assert_eq!(activation.admission_status(), AdmissionStatus::Admitted);
    assert!(activation.issues().is_empty());
    assert_eq!(activation.selected_model_role(), Some(MODEL_ROLE));
    assert_eq!(activation.prompt_fragments(), manifest.prompt_fragments());
    assert_eq!(
        activation.effective_capabilities(),
        &[Text::from(CAPABILITY)]
    );
    assert_eq!(activation.effective_tools(), &[Text::from(TOOLSET)]);
    assert_eq!(
        activation.effective_memory_scopes(),
        &[Text::from(MEMORY_SCOPE)]
    );
    assert_eq!(
        activation.effective_context_reference_scopes(),
        &[Text::from(CONTEXT_SCOPE)]
    );
    assert_eq!(
        activation.effective_workflow_abilities(),
        &[Text::from(WORKFLOW_ABILITY)]
    );
    assert!(activation.denied_escalations().is_empty());
}

#[test]
fn safe_receipt_records_metadata_hashes_and_redactions_only() {
    let manifest_hash = safe_hash();
    let policy_hash = safe_hash();
    let receipt = Receipt::new(ReceiptDraft {
        definition_name: NAME,
        source_scope: SourceScope::User,
        trust_status: TrustStatus::Trusted,
        admission_status: AdmissionStatus::NotRequired,
        capability_ceilings: &[CAPABILITY],
        selected_model_role: Some(MODEL_ROLE),
        manifest_hash: &manifest_hash,
        policy_hash: &policy_hash,
        redaction_markers: &[REDACTION_MARKER, SECOND_REDACTION_MARKER],
    })
    .expect("safe receipt metadata should validate");

    assert_eq!(receipt.definition_name(), NAME);
    assert_eq!(receipt.source_scope(), SourceScope::User);
    assert_eq!(receipt.trust_status(), TrustStatus::Trusted);
    assert_eq!(receipt.admission_status(), AdmissionStatus::NotRequired);
    assert_eq!(receipt.capability_ceilings(), &[Text::from(CAPABILITY)]);
    assert_eq!(receipt.selected_model_role(), Some(MODEL_ROLE));
    assert_eq!(receipt.manifest_hash().as_str(), VALID_HASH);
    assert_eq!(receipt.policy_hash().as_str(), VALID_HASH);
    assert_eq!(
        receipt.redaction_markers(),
        &[
            Text::from(SECOND_REDACTION_MARKER),
            Text::from(REDACTION_MARKER)
        ]
    );
    validate_text(SAFE_RECEIPT_TEXT).expect("safe receipt should pass leak scan");
}
