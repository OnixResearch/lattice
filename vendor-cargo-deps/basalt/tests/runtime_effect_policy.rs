// r[verify contract_backends.runtime_effect_permissions.policy]
// r[verify contract_backends.runtime_effect_permissions.decision]
// r[verify contract_backends.runtime_effect_permissions.authority]
// r[verify contract_backends.runtime_effect_permissions.consumer]
// r[verify contract_backends.runtime_effect_permissions.evidence]
// r[verify contract_backends.runtime_effect_permissions.verification]

use basalt::runtime_effect::{
    ApprovalDisposition, DEFAULT_RUNTIME_EFFECT_POLICY_JSON, EffectAuthorityInput, EffectOperation,
    EffectPermissionDiagnostic, EffectRequest, EffectRequestError, EffectScope,
    RUNTIME_EFFECT_CONSUMER_MAPPINGS_JSON, RuntimeEffectPolicyError, evaluate_effect_permission,
    parse_runtime_effect_policy,
};
use serde_json::Value;

const GENERATION: u64 = 1;
const STALE_GENERATION: u64 = GENERATION + 1;
const RULE_PRIORITY: u64 = 100;
const EXPECTED_ANIMUS_MAPPINGS: usize = 3;
const EXPECTED_MOLTEN_MAPPINGS: usize = 2;
const ANIMUS_REVISION: &str = "4f08f89d652a6e6c83b2a41fd9d3c7651daf5d68";
const MOLTEN_REVISION: &str = "4734913c1c230a35fc121b948cdf54ac0619dd2a";
const RAW_SUBJECT: &str = "cap://workspace/input";

const fn no_authority(approval: ApprovalDisposition) -> EffectAuthorityInput<'static> {
    EffectAuthorityInput {
        contract_decision: None,
        verified_ucan: None,
        approval,
    }
}

fn parse_mutated(
    value: &Value,
) -> Result<basalt::runtime_effect::RuntimeEffectPolicy, RuntimeEffectPolicyError> {
    parse_runtime_effect_policy(&serde_json::to_string(value).expect("policy JSON"))
}

#[test]
fn malformed_unknown_duplicate_stale_and_raw_policy_inputs_fail_closed() {
    let mut unknown_scope: Value =
        serde_json::from_str(DEFAULT_RUNTIME_EFFECT_POLICY_JSON).expect("checked policy JSON");
    unknown_scope["rules"][0]["scope"] = Value::String("unknown".to_owned());
    assert_eq!(
        parse_mutated(&unknown_scope),
        Err(RuntimeEffectPolicyError::Malformed)
    );

    let mut unknown_operation: Value =
        serde_json::from_str(DEFAULT_RUNTIME_EFFECT_POLICY_JSON).expect("checked policy JSON");
    unknown_operation["rules"][0]["operation"] = Value::String("execute".to_owned());
    assert_eq!(
        parse_mutated(&unknown_operation),
        Err(RuntimeEffectPolicyError::Malformed)
    );

    let mut duplicate: Value =
        serde_json::from_str(DEFAULT_RUNTIME_EFFECT_POLICY_JSON).expect("checked policy JSON");
    let duplicate_rule = duplicate["rules"][0].clone();
    duplicate["rules"]
        .as_array_mut()
        .expect("rules array")
        .push(duplicate_rule);
    assert_eq!(
        parse_mutated(&duplicate),
        Err(RuntimeEffectPolicyError::DuplicateRule)
    );

    let mut raw_path: Value =
        serde_json::from_str(DEFAULT_RUNTIME_EFFECT_POLICY_JSON).expect("checked policy JSON");
    raw_path["rules"][0]["subject_pattern"] = Value::String("/home/user/secret".to_owned());
    assert_eq!(
        parse_mutated(&raw_path),
        Err(RuntimeEffectPolicyError::InvalidSubject)
    );

    let mut stale: Value =
        serde_json::from_str(DEFAULT_RUNTIME_EFFECT_POLICY_JSON).expect("checked policy JSON");
    stale["reference"]["revision"] = Value::String("0".repeat(ANIMUS_REVISION.len()));
    assert_eq!(
        parse_mutated(&stale),
        Err(RuntimeEffectPolicyError::ReferenceMismatch)
    );

    let mut unsafe_default: Value =
        serde_json::from_str(DEFAULT_RUNTIME_EFFECT_POLICY_JSON).expect("checked policy JSON");
    unsafe_default["default_decision"] = Value::String("allow".to_owned());
    assert_eq!(
        parse_mutated(&unsafe_default),
        Err(RuntimeEffectPolicyError::UnsafeDefault)
    );
}

#[test]
fn ambiguous_stale_missing_and_denied_rules_do_not_allow() {
    let mut ambiguous: Value =
        serde_json::from_str(DEFAULT_RUNTIME_EFFECT_POLICY_JSON).expect("checked policy JSON");
    ambiguous["rules"]
        .as_array_mut()
        .expect("rules array")
        .push(serde_json::json!({
            "rule_id": "filesystem-read-input-deny",
            "scope": "filesystem",
            "operation": "read",
            "match_mode": "exact",
            "subject_pattern": RAW_SUBJECT,
            "priority": RULE_PRIORITY,
            "decision": "deny"
        }));
    let ambiguous = parse_mutated(&ambiguous).expect("ambiguous policy remains structurally valid");
    let request = EffectRequest::new(
        EffectScope::Filesystem,
        EffectOperation::Read,
        RAW_SUBJECT,
        GENERATION,
        true,
        true,
    )
    .expect("normalized request");
    let decision = evaluate_effect_permission(
        &ambiguous,
        &request,
        &no_authority(ApprovalDisposition::Approved),
    );
    assert!(!decision.is_allowed());
    assert_eq!(
        decision.receipt().diagnostic(),
        EffectPermissionDiagnostic::AmbiguousRules
    );

    let policy =
        parse_runtime_effect_policy(DEFAULT_RUNTIME_EFFECT_POLICY_JSON).expect("checked policy");
    let stale = EffectRequest::new(
        EffectScope::Filesystem,
        EffectOperation::Read,
        RAW_SUBJECT,
        STALE_GENERATION,
        true,
        true,
    )
    .expect("normalized stale request");
    assert_eq!(
        evaluate_effect_permission(
            &policy,
            &stale,
            &no_authority(ApprovalDisposition::Approved),
        )
        .receipt()
        .diagnostic(),
        EffectPermissionDiagnostic::StaleProfile
    );

    assert_eq!(
        evaluate_effect_permission(
            &policy,
            &request,
            &no_authority(ApprovalDisposition::Approved),
        )
        .receipt()
        .diagnostic(),
        EffectPermissionDiagnostic::ContractMissing
    );

    let no_match = EffectRequest::new(
        EffectScope::Filesystem,
        EffectOperation::Read,
        "cap://workspace/unlisted",
        GENERATION,
        true,
        true,
    )
    .expect("normalized unlisted request");
    let denied = evaluate_effect_permission(
        &policy,
        &no_match,
        &no_authority(ApprovalDisposition::Approved),
    );
    assert_eq!(
        denied.receipt().diagnostic(),
        EffectPermissionDiagnostic::NoMatchingRule
    );
    let serialized = serde_json::to_string(denied.receipt()).expect("receipt JSON");
    assert!(!serialized.contains("cap://workspace/unlisted"));
}

#[test]
fn request_admission_rejects_wrong_scope_and_raw_native_subjects() {
    assert_eq!(
        EffectRequest::new(
            EffectScope::Network,
            EffectOperation::Read,
            "net://api.example.invalid:443",
            GENERATION,
            true,
            true,
        ),
        Err(EffectRequestError::OperationScopeMismatch)
    );
    assert_eq!(
        EffectRequest::new(
            EffectScope::Filesystem,
            EffectOperation::Read,
            "/home/user/secret",
            GENERATION,
            true,
            true,
        ),
        Err(EffectRequestError::InvalidSubject)
    );
}

#[test]
fn consumer_fixtures_pin_exact_animus_and_molten_cohorts() {
    let mappings: Value =
        serde_json::from_str(RUNTIME_EFFECT_CONSUMER_MAPPINGS_JSON).expect("consumer mapping JSON");
    assert_eq!(mappings["animus"]["revision"], ANIMUS_REVISION);
    assert_eq!(mappings["molten"]["revision"], MOLTEN_REVISION);
    assert_eq!(
        mappings["animus"]["mappings"]
            .as_array()
            .expect("Animus mappings")
            .len(),
        EXPECTED_ANIMUS_MAPPINGS
    );
    assert_eq!(
        mappings["molten"]["mappings"]
            .as_array()
            .expect("Molten mappings")
            .len(),
        EXPECTED_MOLTEN_MAPPINGS
    );
    let serialized = serde_json::to_string(&mappings).expect("mapping JSON");
    assert!(serialized.contains("fixtures do not prove native enforcement or effect execution"));
}
