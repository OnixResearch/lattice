use serde::Deserialize;

const PROFILE_NAME: &str = "default-loop";
const SCRIPT_LABEL: &str = "scripts/default-loop.scm";
const RALPH_PROFILE_NAME: &str = "ralph-loop";
const RALPH_SCRIPT_LABEL: &str = "scripts/ralph-loop.scm";
const BAD_PROFILE_NAME: &str = "Bad Profile";
const PATH_ESCAPE_LABEL: &str = "../outside.scm";
const UNKNOWN_ACTION: &str = "filesystem";
const UNSUPPORTED_FALLBACK: &str = "continue";
const UNSUPPORTED_REDACTION: &str = "raw";
const INVALID_HASH: &str = "not-a-blake3-hash";
const ZERO_TURN_BUDGET: u32 = 0;
const ONE_TURN_BUDGET: u32 = 1;
const TWO_TURN_BUDGET: u32 = 2;
const RALPH_TURN_BUDGET: u32 = 10;
const RAW_PROMPT_TEXT: &str = "prompt: deploy credentials";
const RAW_PROVIDER_PAYLOAD: &str = "provider payload body";
const SCRIPT_SOURCE: &str = r"
(define (animus-decide)
  (animus-model animus-summary))
";
const OTHER_SCRIPT_SOURCE: &str = r"
(define (animus-decide)
  (animus-stop))
";
const EXPORTED_POLICY_JSON: &str = include_str!("../../../../policy/steel-runtime/generated/steel-runtime-policy.json");
const EXPORTED_DEFAULT_SCRIPT: &str = include_str!("../../../../policy/steel-runtime/scripts/default-loop.scm");
const EXPORTED_RALPH_SCRIPT: &str = include_str!("../../../../policy/steel-runtime/scripts/ralph-loop.scm");
const POLICY_SCHEMA: &str = "animus.steel-runtime-policy.v1";
const EXPECTED_PROFILE_COUNT: usize = 2;
const EXPECTED_DEFAULT_PROFILE_COUNT: usize = 1;
const RALPH_TASK_STATE: &str = "branch: ralph/example\nstory: S1 passes false";
const RALPH_INCOMPLETE_MESSAGE: &str = "implemented S1; S2 remains";
const RALPH_COMPLETION_PROMISE: &str = "COMPLETE";
const RALPH_COMPLETE_MESSAGE: &str = "COMPLETE";
const RALPH_INCIDENTAL_COMPLETE_MESSAGE: &str = "not COMPLETE; checks remain";
const RALPH_PROMPT_MARKER: &str = "complete one unfinished Ralph story";

#[test]
fn generated_policy_export_allows_default_script() {
    let policy = exported_policy();
    let profile = default_exported_profile(&policy);
    let action_refs = action_refs(profile);
    let draft = super::ProfileDraft {
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
    let script = super::Script::new(checked_profile.name(), checked_profile.script_label(), EXPORTED_DEFAULT_SCRIPT);
    let activation = super::Activation::new(script, ONE_TURN_BUDGET, super::DEFAULT_RUNTIME_HOST_ACTIONS);
    let receipt = checked_profile.activate(&activation);

    assert_eq!(policy.schema, POLICY_SCHEMA);
    assert_eq!(policy.profiles.len(), EXPECTED_PROFILE_COUNT);
    assert_eq!(profile.name, PROFILE_NAME);
    assert_eq!(profile.script.label, SCRIPT_LABEL);
    assert_eq!(profile.script.hash, super::script_hash(EXPORTED_DEFAULT_SCRIPT));
    assert!(receipt.is_allowed());
    assert_eq!(receipt.denied_reason(), None);
}

#[test]
fn generated_policy_export_allows_ralph_script() {
    let policy = exported_policy();
    let profile = exported_profile_by_name(&policy, RALPH_PROFILE_NAME);
    let action_refs = action_refs(profile);
    let draft = super::ProfileDraft {
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
    let checked_profile = profile_from_draft(&draft).expect("ralph policy should build");
    let script = super::Script::new(checked_profile.name(), checked_profile.script_label(), EXPORTED_RALPH_SCRIPT);
    let activation = super::Activation::new(script, RALPH_TURN_BUDGET, super::DEFAULT_RUNTIME_HOST_ACTIONS);
    let receipt = checked_profile.activate(&activation);

    assert_eq!(profile.name, RALPH_PROFILE_NAME);
    assert_eq!(profile.script.label, RALPH_SCRIPT_LABEL);
    assert_eq!(profile.budget.max_turns, RALPH_TURN_BUDGET);
    assert_eq!(profile.script.hash, super::script_hash(EXPORTED_RALPH_SCRIPT));
    assert!(receipt.is_allowed());
    assert_eq!(receipt.denied_reason(), None);
}

#[test]
fn ralph_script_reprompts_until_completion_promise() {
    let first_turn = crate::LoopTurn::new(RALPH_TASK_STATE, None, None).expect("ralph first turn should validate");
    let first_decision = crate::SteelLoopFabric::decide_once(EXPORTED_RALPH_SCRIPT, &first_turn)
        .expect("ralph first turn should produce model request");
    assert_model_prompt_contains(first_decision, &[RALPH_TASK_STATE, RALPH_PROMPT_MARKER, RALPH_COMPLETION_PROMISE]);

    let incomplete_turn = crate::LoopTurn::new(RALPH_TASK_STATE, Some(RALPH_INCOMPLETE_MESSAGE), None)
        .expect("ralph incomplete turn should validate");
    let incomplete_decision = crate::SteelLoopFabric::decide_once(EXPORTED_RALPH_SCRIPT, &incomplete_turn)
        .expect("ralph incomplete turn should produce another model request");
    assert_model_prompt_contains(incomplete_decision, &["Previous iteration summary", RALPH_INCOMPLETE_MESSAGE]);

    let incidental_turn = crate::LoopTurn::new(RALPH_TASK_STATE, Some(RALPH_INCIDENTAL_COMPLETE_MESSAGE), None)
        .expect("ralph incidental turn should validate");
    let incidental_decision = crate::SteelLoopFabric::decide_once(EXPORTED_RALPH_SCRIPT, &incidental_turn)
        .expect("incidental COMPLETE must reprompt");
    assert_model_prompt_contains(incidental_decision, &[
        "Previous iteration summary",
        RALPH_INCIDENTAL_COMPLETE_MESSAGE,
    ]);

    let complete_turn = crate::LoopTurn::new(RALPH_TASK_STATE, Some(RALPH_COMPLETE_MESSAGE), None)
        .expect("ralph complete turn should validate");
    let complete_decision = crate::SteelLoopFabric::decide_once(EXPORTED_RALPH_SCRIPT, &complete_turn)
        .expect("ralph complete turn should produce a candidate");

    match complete_decision {
        crate::LoopDecision::CompletionCandidate { candidate } => {
            assert_eq!(candidate.request(), RALPH_COMPLETE_MESSAGE);
        }
        other => panic!("expected ralph completion candidate, got {other:?}"),
    }
}

#[test]
fn valid_profile_allows_matching_activation() {
    let profile = valid_profile();
    let receipt = profile.activate(&valid_activation(&profile));

    assert_eq!(receipt.schema(), super::digest::receipt_schema());
    assert_eq!(receipt.status(), super::ActivationStatus::Allowed);
    assert!(receipt.is_allowed());
    assert_eq!(receipt.profile(), PROFILE_NAME);
    assert_eq!(receipt.script_hash(), super::script_hash(SCRIPT_SOURCE));
    assert_eq!(receipt.profile_hash(), profile.profile_hash());
    assert_eq!(receipt.denied_reason(), None);
    assert_eq!(receipt.fallback_mode(), super::FallbackMode::Block);
    assert_eq!(receipt.redaction(), super::ReceiptRedaction::HashOnly);
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

#[test]
fn bad_profile_name_is_rejected() {
    let error = profile_with_name(BAD_PROFILE_NAME).expect_err("bad profile name should fail");

    assert_eq!(error.reason(), super::DenyReason::BadProfileName);
    assert_eq!(error.field(), "name");
}

#[test]
fn non_positive_budget_is_rejected() {
    let error = profile_with_budget(ZERO_TURN_BUDGET).expect_err("zero budget should fail");

    assert_eq!(error.reason(), super::DenyReason::InvalidBudget);
}

#[test]
fn unknown_host_action_is_rejected() {
    let error = profile_with_actions(&["model", UNKNOWN_ACTION]).expect_err("unknown action should fail");

    assert_eq!(error.reason(), super::DenyReason::UnknownHostAction);
}

#[test]
fn path_escape_is_rejected() {
    let error = profile_with_script_label(PATH_ESCAPE_LABEL).expect_err("path escape should fail");

    assert_eq!(error.reason(), super::DenyReason::PathEscape);
}

#[test]
fn unsupported_fallback_is_rejected() {
    let error = profile_with_fallback(UNSUPPORTED_FALLBACK).expect_err("fallback should fail");

    assert_eq!(error.reason(), super::DenyReason::UnsupportedFallback);
}

#[test]
fn unsupported_redaction_is_rejected() {
    let error = profile_with_redaction(UNSUPPORTED_REDACTION).expect_err("redaction should fail");

    assert_eq!(error.reason(), super::DenyReason::UnsupportedReceiptRedaction);
}

#[test]
fn invalid_hash_is_rejected() {
    let error = profile_with_hash(INVALID_HASH).expect_err("invalid hash should fail");

    assert_eq!(error.reason(), super::DenyReason::HashMismatch);
}

#[test]
fn missing_script_denies_activation() {
    let profile = valid_profile();
    let script = super::Script::new(PROFILE_NAME, SCRIPT_LABEL, "");
    let receipt = activate_script(&profile, script, ONE_TURN_BUDGET);

    assert_eq!(receipt.status(), super::ActivationStatus::Denied);
    assert_eq!(receipt.denied_reason(), Some(super::DenyReason::MissingScript));
}

#[test]
fn hash_mismatch_denies_activation() {
    let profile = valid_profile();
    let script = super::Script::new(PROFILE_NAME, SCRIPT_LABEL, OTHER_SCRIPT_SOURCE);
    let receipt = activate_script(&profile, script, ONE_TURN_BUDGET);

    assert_eq!(receipt.status(), super::ActivationStatus::Denied);
    assert_eq!(receipt.denied_reason(), Some(super::DenyReason::HashMismatch));
}

#[test]
fn oversized_budget_denies_activation() {
    let profile = valid_profile();
    let script = super::Script::new(PROFILE_NAME, SCRIPT_LABEL, SCRIPT_SOURCE);
    let receipt = activate_script(&profile, script, TWO_TURN_BUDGET);

    assert_eq!(receipt.status(), super::ActivationStatus::Denied);
    assert_eq!(receipt.denied_reason(), Some(super::DenyReason::InvalidBudget));
}

#[test]
fn disallowed_action_denies_activation() {
    let profile = profile_with_actions(&["model"]).expect("model-only profile should build");
    let receipt = profile.activate(&valid_activation(&profile));

    assert_eq!(receipt.status(), super::ActivationStatus::Denied);
    assert_eq!(receipt.denied_reason(), Some(super::DenyReason::HostActionDenied));
}

#[test]
fn mismatched_profile_denies_activation() {
    let profile = valid_profile();
    let script = super::Script::new("other-loop", SCRIPT_LABEL, SCRIPT_SOURCE);
    let receipt = activate_script(&profile, script, ONE_TURN_BUDGET);

    assert_eq!(receipt.status(), super::ActivationStatus::Denied);
    assert_eq!(receipt.denied_reason(), Some(super::DenyReason::ProfileMismatch));
}

#[test]
fn path_escape_denies_activation() {
    let profile = valid_profile();
    let script = super::Script::new(PROFILE_NAME, PATH_ESCAPE_LABEL, SCRIPT_SOURCE);
    let receipt = activate_script(&profile, script, ONE_TURN_BUDGET);

    assert_eq!(receipt.status(), super::ActivationStatus::Denied);
    assert_eq!(receipt.denied_reason(), Some(super::DenyReason::PathEscape));
}

#[derive(Deserialize)]
struct ExportedPolicy {
    schema: String,
    profiles: Vec<ExportedProfile>,
}

#[derive(Deserialize)]
struct ExportedProfile {
    name: String,
    enabled: bool,
    #[serde(rename = "default")]
    is_default: bool,
    script: ExportedScript,
    budget: ExportedBudget,
    allowed_host_actions: Vec<String>,
    fallback_mode: String,
    receipt_redaction: ExportedReceiptRedaction,
}

#[derive(Deserialize)]
struct ExportedScript {
    label: String,
    hash: String,
}

#[derive(Deserialize)]
struct ExportedBudget {
    max_turns: u32,
}

#[derive(Deserialize)]
struct ExportedReceiptRedaction {
    mode: String,
}

fn exported_policy() -> ExportedPolicy {
    serde_json::from_str(EXPORTED_POLICY_JSON).expect("generated Steel runtime policy should parse")
}

fn default_exported_profile(policy: &ExportedPolicy) -> &ExportedProfile {
    let default_profiles: Vec<&ExportedProfile> = policy.profiles.iter().filter(|profile| profile.is_default).collect();

    assert_eq!(default_profiles.len(), EXPECTED_DEFAULT_PROFILE_COUNT);
    default_profiles[0]
}

fn exported_profile_by_name<'a>(policy: &'a ExportedPolicy, name: &str) -> &'a ExportedProfile {
    policy
        .profiles
        .iter()
        .find(|profile| profile.name.as_str() == name)
        .expect("exported profile should exist")
}

fn assert_model_prompt_contains(decision: crate::LoopDecision, needles: &[&str]) {
    match decision {
        crate::LoopDecision::CallModel { request } => {
            for needle in needles {
                assert!(request.prompt().contains(needle), "ralph prompt should contain {needle:?}");
            }
        }
        other => panic!("expected ralph model request, got {other:?}"),
    }
}

fn action_refs(profile: &ExportedProfile) -> Vec<&str> {
    profile.allowed_host_actions.iter().map(String::as_str).collect()
}

fn valid_profile() -> super::Profile {
    profile_with_budget(ONE_TURN_BUDGET).expect("valid profile should build")
}

fn valid_activation(profile: &super::Profile) -> super::Activation<'_> {
    let script = super::Script::new(profile.name(), profile.script_label(), SCRIPT_SOURCE);
    super::Activation::new(script, ONE_TURN_BUDGET, super::DEFAULT_RUNTIME_HOST_ACTIONS)
}

fn activate_script(profile: &super::Profile, script: super::Script<'_>, turn_budget: u32) -> super::ActivationReceipt {
    let activation = super::Activation::new(script, turn_budget, super::DEFAULT_RUNTIME_HOST_ACTIONS);
    profile.activate(&activation)
}

fn profile_with_name(name: &str) -> Result<super::Profile, super::Error> {
    with_valid_draft(|draft| profile_from_draft(&super::ProfileDraft { name, ..*draft }))
}

fn profile_with_budget(max_turns: u32) -> Result<super::Profile, super::Error> {
    with_valid_draft(|draft| profile_from_draft(&super::ProfileDraft { max_turns, ..*draft }))
}

fn profile_with_actions(allowed_host_actions: &[&str]) -> Result<super::Profile, super::Error> {
    with_valid_draft(|draft| {
        profile_from_draft(&super::ProfileDraft {
            allowed_host_actions,
            ..*draft
        })
    })
}

fn profile_with_script_label(script_label: &str) -> Result<super::Profile, super::Error> {
    with_valid_draft(|draft| profile_from_draft(&super::ProfileDraft { script_label, ..*draft }))
}

fn profile_with_fallback(fallback_mode: &str) -> Result<super::Profile, super::Error> {
    with_valid_draft(|draft| {
        profile_from_draft(&super::ProfileDraft {
            fallback_mode,
            ..*draft
        })
    })
}

fn profile_with_redaction(receipt_redaction: &str) -> Result<super::Profile, super::Error> {
    with_valid_draft(|draft| {
        profile_from_draft(&super::ProfileDraft {
            receipt_redaction,
            ..*draft
        })
    })
}

fn profile_with_hash(script_hash: &str) -> Result<super::Profile, super::Error> {
    with_valid_draft(|draft| profile_from_draft(&super::ProfileDraft { script_hash, ..*draft }))
}

fn profile_from_draft(draft: &super::ProfileDraft<'_>) -> Result<super::Profile, super::Error> {
    super::Profile::from_draft(draft)
}

fn with_valid_draft(
    build: impl FnOnce(&super::ProfileDraft<'_>) -> Result<super::Profile, super::Error>,
) -> Result<super::Profile, super::Error> {
    let computed_hash = super::script_hash(SCRIPT_SOURCE);
    build(&super::ProfileDraft {
        name: PROFILE_NAME,
        enabled: true,
        is_default: true,
        script_label: SCRIPT_LABEL,
        script_hash: computed_hash.as_str(),
        max_turns: ONE_TURN_BUDGET,
        allowed_host_actions: &["model", "tool", "respond", "completion-candidate", "stop"],
        fallback_mode: "block",
        receipt_redaction: "hash-only",
    })
}
