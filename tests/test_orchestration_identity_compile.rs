// r[verify lattice.agent_orchestration.nominal_identities.types.same_role]
// r[verify lattice.agent_orchestration.nominal_identities.compile_time]
#[test]
fn orchestration_identity_roles_compile_only_with_same_role_values() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/fixtures/orchestration-identities/compile-pass.rs");
    cases.compile_fail("tests/fixtures/orchestration-identities/compile-fail.rs");
}
