// r[verify contract_backends.world_branch_authority.policy]
// r[verify contract_backends.world_branch_authority.decision]
// r[verify contract_backends.world_branch_authority.currentness]
// r[verify contract_backends.world_branch_authority.evidence]
// r[verify contract_backends.world_branch_authority.consumer]
// r[verify contract_backends.world_branch_authority.verification]

use basalt::world_branch_authority::*;

const DIGEST_HEX_LENGTH: usize = 64;
const POLICY_GENERATION: u64 = 1;
const EXPECTED_MAPPING_COUNT: usize = 7;

fn digest(byte: char) -> String {
    format!("blake3:{}", byte.to_string().repeat(DIGEST_HEX_LENGTH))
}

fn scope(resource: &str) -> CapabilityScope {
    CapabilityScope::new(resource, ["read".to_string()], None).expect("valid scope")
}

fn authority() -> AuthorityCurrentness {
    AuthorityCurrentness {
        observation_ref: digest('4'),
        policy: CurrentFact::Current,
        capability: CurrentFact::Current,
        revocation: CurrentFact::Current,
        replay: CurrentFact::Current,
        scope: CurrentFact::Current,
        ucan: UcanComposition::Verified,
    }
}

#[test]
fn generated_policy_and_public_api_admit_one_metadata_only_copy_plan() {
    let policy = parse_branch_authority_policy(DEFAULT_WORLD_BRANCH_AUTHORITY_POLICY_JSON)
        .expect("generated policy");
    let request = BranchAuthorityRequest::new(
        CapabilityClass::PublicArtifact,
        BranchAction::Create,
        digest('1'),
        digest('2'),
        digest('3'),
        scope("cap://artifact"),
        scope("cap://artifact"),
        POLICY_GENERATION,
        true,
    )
    .expect("request");
    let decision = evaluate_branch_authority(&policy, &request, &authority());
    let receipt = branch_authority_receipt(&decision).expect("receipt");
    assert!(decision.is_allowed());
    assert_eq!(decision.mode(), Some(BranchMode::Copyable));
    assert_eq!(
        decision.obligations(),
        [BranchObligation::IndependentDestinationGrant]
    );
    let text = serde_json::to_string(&receipt).expect("receipt JSON");
    assert!(!text.contains("cap://artifact"));
    assert!(!text.contains("bearer-token="));
}

#[test]
fn live_fallback_and_lossy_consumer_mapping_remain_denied() {
    let policy = parse_branch_authority_policy(DEFAULT_WORLD_BRANCH_AUTHORITY_POLICY_JSON)
        .expect("generated policy");
    let request = BranchAuthorityRequest::new(
        CapabilityClass::ExternalEffect,
        BranchAction::Activate,
        digest('1'),
        digest('2'),
        digest('3'),
        scope("cap://external"),
        scope("cap://external"),
        POLICY_GENERATION,
        true,
    )
    .expect("request");
    assert_eq!(
        evaluate_branch_authority(&policy, &request, &authority()).diagnostic(),
        BranchAuthorityDiagnostic::ActionModeMismatch
    );

    let mappings: serde_json::Value =
        serde_json::from_str(WORLD_BRANCH_AUTHORITY_CONSUMER_MAPPINGS_JSON)
            .expect("consumer mappings");
    assert_eq!(
        mappings["molten"]["mappings"]
            .as_array()
            .expect("mapping rows")
            .len(),
        EXPECTED_MAPPING_COUNT
    );
    assert_eq!(mappings["molten"]["policy_owner"], "basalt");
    assert_eq!(mappings["molten"]["enforcement_owner"], "molten");
}
