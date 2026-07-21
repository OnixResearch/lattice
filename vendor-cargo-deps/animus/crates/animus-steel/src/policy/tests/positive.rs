use super::*;

#[test]
fn generated_export_allows_default_script() {
    let document = exported_document();
    let profile = default_exported_profile(&document);
    let action_refs = action_refs(profile);
    let draft = super::super::ProfileDraft {
        name: profile.name.as_str(),
        enabled: profile.enabled,
        is_default: profile.is_default,
        script_label: profile.script.label.as_str(),
        script_hash: profile.script.hash.as_str(),
        max_turns: profile.budget.max_turns,
        allowed_host_actions: action_refs.as_slice(),
        fallback_mode: profile.fallback_mode.as_str(),
        receipt_redaction: profile.receipt_redaction.mode.as_str(),
    };
    let checked_profile = profile_from_draft(&draft).expect("exported policy should build");
    let script = super::super::Script::new(
        checked_profile.name(),
        checked_profile.script_label(),
        EXPORTED_DEFAULT_SCRIPT,
    );
    let activation = super::super::Activation::new(
        script,
        ONE_TURN_BUDGET,
        super::super::DEFAULT_RUNTIME_HOST_ACTIONS,
    );
    let receipt = checked_profile.activate(&activation);

    assert_eq!(document.schema, POLICY_SCHEMA);
    assert_eq!(profile.name, PROFILE_NAME);
    assert_eq!(profile.script.label, SCRIPT_LABEL);
    assert_eq!(
        profile.script.hash,
        super::super::script_hash(EXPORTED_DEFAULT_SCRIPT)
    );
    assert!(receipt.is_allowed());
    assert_eq!(receipt.denied_reason(), None);
}

#[test]
fn valid_profile_allows_matching_activation() {
    let profile = valid_profile();
    let receipt = profile.activate(&valid_activation(&profile));

    assert_eq!(receipt.schema(), super::super::digest::receipt_schema());
    assert_eq!(receipt.status(), super::super::ActivationStatus::Allowed);
    assert!(receipt.is_allowed());
    assert_eq!(receipt.profile(), PROFILE_NAME);
    assert_eq!(
        receipt.script_hash(),
        super::super::script_hash(SCRIPT_SOURCE)
    );
    assert_eq!(receipt.profile_hash(), profile.profile_hash());
    assert_eq!(receipt.denied_reason(), None);
    assert_eq!(receipt.fallback_mode(), super::super::FallbackMode::Block);
    assert_eq!(
        receipt.redaction(),
        super::super::ReceiptRedaction::HashOnly
    );
    assert!(!receipt.receipt_hash().is_empty());
}

#[test]
fn receipt_debug_omits_raw_script_and_payload_text() {
    let profile = valid_profile();
    let receipt = profile.activate(&valid_activation(&profile));
    let rendered = format!("{receipt:?}");

    assert!(!rendered.contains(SCRIPT_SOURCE.trim()));
    assert!(!rendered.contains(RAW_PROMPT_TEXT));
    assert!(!rendered.contains(RAW_PROVIDER_PAYLOAD));
}
