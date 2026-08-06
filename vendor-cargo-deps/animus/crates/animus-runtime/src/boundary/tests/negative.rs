use super::*;

#[test]
fn unmatched_sandbox_escalation_fails_closed_without_raw_command_body() {
    let rules = vec![allow_cargo_rule()];
    let event = event_for(&command(&[SECRET_COMMAND, "--token", SECRET_TOKEN]), rules.as_slice());
    let rendered = format!("{event:?}");

    let crate::RuntimeEvent::SandboxEscalation {
        decision,
        rule_ids,
        command_fingerprints,
        redaction_marker,
        ..
    } = event
    else {
        panic!("expected sandbox escalation event");
    };

    assert_eq!(decision, animus_adapters::ExecPolicyDecision::Forbid);
    assert!(rule_ids.is_empty());
    assert_eq!(command_fingerprints.len(), 1);
    assert_eq!(redaction_marker, "command_hash_only");
    assert!(!rendered.contains(SECRET_COMMAND));
    assert!(!rendered.contains(SECRET_TOKEN));
}

#[test]
fn unsupported_shell_sandbox_escalation_is_opaque_forbid_with_hash_only_evidence() {
    let rules = vec![allow_cargo_rule()];
    let event = event_for(&command(&["sh", "-c", PIPE_WRAPPER_SOURCE]), rules.as_slice());
    let rendered = format!("{event:?}");

    let crate::RuntimeEvent::SandboxEscalation {
        decision,
        rule_ids,
        command_fingerprints,
        redaction_marker,
        ..
    } = event
    else {
        panic!("expected sandbox escalation event");
    };

    assert_eq!(decision, animus_adapters::ExecPolicyDecision::Forbid);
    assert!(rule_ids.is_empty());
    assert_eq!(command_fingerprints.len(), 1);
    assert_eq!(redaction_marker, "command_hash_only");
    assert!(!rendered.contains(PIPE_WRAPPER_SOURCE));
}
