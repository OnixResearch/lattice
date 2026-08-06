use super::*;

#[test]
fn sandbox_escalation_event_records_allow_prompt_and_forbid_outcomes() {
    let rules = vec![allow_cargo_rule(), prompt_nix_rule(), forbid_rm_rule()];
    let allow_event = event_for(&command(&["cargo", "metadata"]), rules.as_slice());
    let prompt_event = event_for(&command(&["nix", "build"]), rules.as_slice());
    let forbid_event = event_for(&command(&["rm", "-rf", "target"]), rules.as_slice());

    assert_eq!(allow_event.kind(), crate::RuntimeEventKind::SandboxEscalation);
    assert_eq!(prompt_event.kind(), crate::RuntimeEventKind::SandboxEscalation);
    assert_eq!(forbid_event.kind(), crate::RuntimeEventKind::SandboxEscalation);
    assert_sandbox_event(&allow_event, animus_adapters::ExecPolicyDecision::Allow, ALLOW_CARGO_ID);
    assert_sandbox_event(&prompt_event, animus_adapters::ExecPolicyDecision::Prompt, PROMPT_NIX_ID);
    assert_sandbox_event(&forbid_event, animus_adapters::ExecPolicyDecision::Forbid, FORBID_RM_ID);
}

#[test]
fn sandbox_escalation_event_has_stable_kind_label() {
    let rules = vec![allow_cargo_rule()];
    let event = event_for(&command(&["cargo", "metadata"]), rules.as_slice());

    assert_eq!(event.kind(), crate::RuntimeEventKind::SandboxEscalation);
    assert_eq!(event.kind().as_str(), "sandbox_escalation");
}

fn assert_sandbox_event(
    event: &crate::RuntimeEvent,
    expected_decision: animus_adapters::ExecPolicyDecision,
    expected_rule_id: &str,
) {
    let crate::RuntimeEvent::SandboxEscalation {
        decision,
        rule_ids,
        command_fingerprints,
        active_sandbox,
        requested_capability,
        escalation_reason,
        redaction_marker,
    } = event
    else {
        panic!("expected sandbox escalation event");
    };

    assert_eq!(*decision, expected_decision);
    assert_eq!(rule_ids, &[String::from(expected_rule_id)]);
    assert_eq!(command_fingerprints.len(), 1);
    assert_eq!(active_sandbox, SANDBOX);
    assert_eq!(requested_capability, CAPABILITY);
    assert_eq!(escalation_reason, REASON);
    assert_eq!(redaction_marker, "command_hash_only");
}
