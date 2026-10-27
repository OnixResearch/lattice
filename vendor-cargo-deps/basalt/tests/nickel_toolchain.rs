// r[verify basalt.nickel_toolchain.boundary]
// r[verify basalt.nickel_toolchain.compatibility]

use basalt::{CapabilityGrant, EnforcementRequest, default_policy, enforce};

const EXACT_NICKEL_VERSION: &str = "1.17.0";
const EXACT_NICKEL_REVISION: &str = "1320a983e6c3d1e2fb53dd2464b084b4903b1426";
const POLICY_CONTRACT: &str = "document-write";
const POLICY_RESOURCE: &str = "did:key:z6Mkresource/docs/nickel-cohort";
const POLICY_ABILITY: &str = "document/write";

#[test]
fn evaluator_identity_is_exact_and_policy_still_requires_runtime_authority() {
    assert_eq!(EXACT_NICKEL_VERSION, "1.17.0");
    assert_eq!(
        EXACT_NICKEL_REVISION,
        "1320a983e6c3d1e2fb53dd2464b084b4903b1426"
    );

    let policy = default_policy().expect("Nickel-exported policy must parse");
    let request = EnforcementRequest::new(POLICY_CONTRACT, POLICY_RESOURCE, POLICY_ABILITY);
    let denied = enforce(&policy, &request).expect("runtime enforcement must return a receipt");
    assert!(!denied.is_allowed());

    let allowed_request =
        request.with_capability(CapabilityGrant::new(POLICY_RESOURCE, POLICY_ABILITY));
    let allowed =
        enforce(&policy, &allowed_request).expect("exact runtime authority must evaluate");
    assert!(allowed.is_allowed());
}
