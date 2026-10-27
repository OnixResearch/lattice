const PROFILE_ID: &str = "code-review";
const PROFILE_SOURCE: &str = "harness:profiles/code-review";
const PROFILE_PURPOSE: &str = "review a change safely";
const PROFILE_BODY: &str = "#!animus mode=review chain=plan\nReview the diff and cite risks.";
const BASE_PRECEDENCE: u16 = 10;
const OVERSIZED_PRECEDENCE: u16 = super::support::MAX_PRECEDENCE.saturating_add(1);

fn profile_draft(
    source_trust: super::types::SourceTrust,
) -> super::types::DescriptorDraft<'static> {
    super::types::DescriptorDraft {
        profile_id: PROFILE_ID,
        purpose: PROFILE_PURPOSE,
        action_posture: super::types::Mode::Review,
        required_capabilities: &[super::types::Capability::Read],
        source_identity: PROFILE_SOURCE,
        trust_scope: super::types::TrustScope::Harness,
        source_trust,
        precedence: BASE_PRECEDENCE,
        body: PROFILE_BODY,
        inclusion_policy: super::types::InclusionPolicy::IncludeWhenTrusted,
    }
}

fn descriptor(source_trust: super::types::SourceTrust) -> super::types::Descriptor {
    super::types::Descriptor::from_draft(profile_draft(source_trust)).expect("valid profile")
}

#[test]
fn descriptor_accepts_valid_data() {
    let profile = descriptor(super::types::SourceTrust::Trusted);

    assert_eq!(profile.profile_id(), PROFILE_ID);
    assert_eq!(profile.purpose(), PROFILE_PURPOSE);
    assert_eq!(profile.action_posture(), super::types::Mode::Review);
    assert_eq!(
        profile.required_capabilities(),
        &[super::types::Capability::Read]
    );
    assert_eq!(profile.source_identity(), PROFILE_SOURCE);
    assert_eq!(profile.precedence(), BASE_PRECEDENCE);
}

#[test]
fn descriptor_rejects_empty_capabilities() {
    let mut draft = profile_draft(super::types::SourceTrust::Trusted);
    draft.required_capabilities = &[];

    let error = super::types::Descriptor::from_draft(draft).expect_err("missing capabilities");

    assert_eq!(error, super::types::Error::MissingCapabilities);
}

#[test]
fn descriptor_rejects_out_of_range_precedence() {
    let mut draft = profile_draft(super::types::SourceTrust::Trusted);
    draft.precedence = OVERSIZED_PRECEDENCE;

    let error = super::types::Descriptor::from_draft(draft).expect_err("bad precedence");

    assert_eq!(
        error,
        super::types::Error::PrecedenceOutOfRange {
            min: super::support::MIN_PRECEDENCE,
            max: super::support::MAX_PRECEDENCE,
            actual: OVERSIZED_PRECEDENCE,
        }
    );
}

#[test]
fn assembly_includes_trusted_profile_and_strips_directive_line() {
    let profile = descriptor(super::types::SourceTrust::Trusted);
    let assembly = super::compose::assemble(
        &[profile],
        super::compose::AssemblyPolicy::default(),
        &[super::types::Mode::Plan],
    );

    assert_eq!(assembly.segments().len(), 1);
    assert_eq!(assembly.segments()[0].profile_id(), PROFILE_ID);
    assert_eq!(assembly.segments()[0].posture(), super::types::Mode::Review);
    assert_eq!(
        assembly.segments()[0].text(),
        "Review the diff and cite risks."
    );
    assert!(assembly.diagnostics().is_empty());
    assert_eq!(
        assembly.receipts()[0].redaction(),
        super::evidence::ReceiptRedaction::BodyHashed
    );
}

#[test]
fn assembly_omits_untrusted_profile_by_default() {
    let profile = descriptor(super::types::SourceTrust::Untrusted);
    let assembly = super::compose::assemble(
        &[profile],
        super::compose::AssemblyPolicy::default(),
        &[super::types::Mode::Plan],
    );

    assert!(assembly.segments().is_empty());
    assert_eq!(assembly.diagnostics().len(), 1);
    assert_eq!(
        assembly.diagnostics()[0].reason(),
        super::compose::OmittedReason::UntrustedSource
    );
    assert_eq!(
        assembly.receipts()[0].omitted_reason(),
        Some(super::compose::OmittedReason::UntrustedSource)
    );
}

#[test]
fn directives_parse_known_mode_and_supported_chain_target() {
    let parsed_directives = super::directives::parse(PROFILE_BODY, &[super::types::Mode::Plan]);

    assert_eq!(parsed_directives.mode(), Some(super::types::Mode::Review));
    assert_eq!(
        parsed_directives.chain_target(),
        Some(super::types::Mode::Plan)
    );
    assert_eq!(
        parsed_directives.visible_body(),
        "Review the diff and cite risks."
    );
    assert!(parsed_directives.diagnostics().is_empty());
}

#[test]
fn directives_fail_closed_for_unknown_mode() {
    let parsed_directives =
        super::directives::parse("#!animus mode=superuser\nBody", &[super::types::Mode::Plan]);

    assert_eq!(parsed_directives.mode(), None);
    assert_eq!(parsed_directives.chain_target(), None);
    assert_eq!(parsed_directives.diagnostics().len(), 1);
    assert_eq!(
        parsed_directives.diagnostics()[0].issue(),
        super::directives::Issue::UnknownMode
    );
}

#[test]
fn directives_fail_closed_for_malformed_and_duplicate_directives() {
    let parsed_directives = super::directives::parse(
        "#!animus mode=plan mode=code chain\nBody",
        &[super::types::Mode::Review],
    );

    assert_eq!(parsed_directives.mode(), None);
    assert_eq!(parsed_directives.chain_target(), None);
    assert_eq!(parsed_directives.diagnostics().len(), 2);
    assert_eq!(
        parsed_directives.diagnostics()[0].issue(),
        super::directives::Issue::DuplicateDirective
    );
    assert_eq!(
        parsed_directives.diagnostics()[1].issue(),
        super::directives::Issue::MalformedToken
    );
}

#[test]
fn directives_fail_closed_for_unsupported_chain_target() {
    let parsed_directives = super::directives::parse(
        "#!animus mode=plan chain=code\nBody",
        &[super::types::Mode::Review],
    );

    assert_eq!(parsed_directives.mode(), None);
    assert_eq!(parsed_directives.chain_target(), None);
    assert_eq!(
        parsed_directives.diagnostics()[0].issue(),
        super::directives::Issue::UnsupportedChainTarget
    );
}

#[test]
fn mode_binding_cannot_grant_missing_write_authority() {
    let binding = super::authority::bind_mode(super::authority::BindingRequest {
        requested_mode: super::types::Mode::Code,
        requested_capabilities: &[super::types::Capability::Write],
        host_allowed_capabilities: &[super::types::Capability::Read],
        policy_allowed_capabilities: &[
            super::types::Capability::Read,
            super::types::Capability::Write,
        ],
    });

    assert_eq!(binding.requested_mode(), super::types::Mode::Code);
    assert!(!binding.all_allowed());
    assert_eq!(
        binding.bindings()[0].status(),
        super::authority::BindingStatus::DeniedByHostAuthority
    );
}

#[test]
fn mode_binding_denies_policy_omitted_exec_network_model_and_tool() {
    let binding = super::authority::bind_mode(super::authority::BindingRequest {
        requested_mode: super::types::Mode::Code,
        requested_capabilities: &[
            super::types::Capability::Exec,
            super::types::Capability::Network,
            super::types::Capability::Model,
            super::types::Capability::Tool,
        ],
        host_allowed_capabilities: &[
            super::types::Capability::Exec,
            super::types::Capability::Network,
            super::types::Capability::Model,
            super::types::Capability::Tool,
        ],
        policy_allowed_capabilities: &[super::types::Capability::Read],
    });

    assert_eq!(binding.bindings().len(), 4);
    assert!(
        binding
            .bindings()
            .iter()
            .all(|binding| binding.status()
                == super::authority::BindingStatus::DeniedByRuntimePolicy)
    );
}

#[test]
fn chain_plan_accepts_transition_with_bounded_instruction() {
    let plan = super::continuity::ChainPlan::from_request(super::continuity::ChainRequest {
        source_profile_id: "brainstorm",
        target_profile_id: "plan",
        decision: super::continuity::ChainDecision::Accepted,
        extra_instruction: Some("Preserve acceptance criteria."),
    })
    .expect("valid chain plan");

    assert_eq!(plan.source_profile_id(), "brainstorm");
    assert_eq!(plan.target_profile_id(), "plan");
    assert_eq!(plan.active_profile_id(), "plan");
    assert_eq!(
        plan.extra_instruction(),
        Some("Preserve acceptance criteria.")
    );
}

#[test]
fn chain_plan_decline_keeps_source_profile_active() {
    let plan = super::continuity::ChainPlan::from_request(super::continuity::ChainRequest {
        source_profile_id: "code",
        target_profile_id: "review",
        decision: super::continuity::ChainDecision::Declined,
        extra_instruction: None,
    })
    .expect("valid declined plan");

    assert_eq!(plan.active_profile_id(), "code");
    assert_eq!(plan.decision(), super::continuity::ChainDecision::Declined);
}

#[test]
fn chain_plan_rejects_unbounded_extra_instruction() {
    let long_instruction =
        "x".repeat(super::support::MAX_EXTRA_INSTRUCTION_CHARS.saturating_add(1));

    let error = super::continuity::ChainPlan::from_request(super::continuity::ChainRequest {
        source_profile_id: "code",
        target_profile_id: "review",
        decision: super::continuity::ChainDecision::Accepted,
        extra_instruction: Some(long_instruction.as_str()),
    })
    .expect_err("long chain instruction rejected");

    assert_eq!(
        error,
        super::types::Error::ExtraInstructionTooLong {
            max_chars: super::support::MAX_EXTRA_INSTRUCTION_CHARS,
            actual_chars: long_instruction.chars().count(),
        }
    );
}

#[test]
fn receipt_uses_hash_without_raw_profile_body() {
    let profile = descriptor(super::types::SourceTrust::Trusted);
    let parsed_directives = super::directives::parse(profile.body(), &[super::types::Mode::Plan]);
    let receipt = super::evidence::Receipt::from_profile(&profile, &parsed_directives, None, None);

    assert_eq!(receipt.profile_id(), PROFILE_ID);
    assert_eq!(receipt.source_identity(), PROFILE_SOURCE);
    assert_eq!(receipt.body_hash().len(), super::support::HASH_HEX_CHARS);
    assert!(!receipt.body_hash().contains("Review the diff"));
    assert_eq!(receipt.directive_mode(), Some(super::types::Mode::Review));
    assert_eq!(
        receipt.directive_chain_target(),
        Some(super::types::Mode::Plan)
    );
    assert_eq!(receipt.directive_issue_count(), 0);
}
