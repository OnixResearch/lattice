// r[verify lattice.execution_backend_orchestration.contract]
// r[verify lattice.execution_backend_orchestration.planner_mapping]
// r[verify lattice.execution_backend_orchestration.admission]
// r[verify lattice.execution_backend_orchestration.persistence]
// r[verify lattice.execution_backend_orchestration.dispatch]
// r[verify lattice.execution_backend_orchestration.inspection]
// r[verify lattice.execution_backend_orchestration.verification]

const PLANNER_REVISION: &str = "54b2fdaf57ac9bbc34e6c73f75d803d625f84ec0";
const POSITIVE_FIXTURE: &str = "tests/fixtures/execution-backends/plan-guarded-local.ncl";
const NEGATIVE_FIXTURE: &str = "tests/fixtures/execution-backends/plan-stale-cohort.ncl";

#[test]
fn frozen_animus_planner_fixtures_accept_exact_and_reject_stale_cohorts() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    lattice::operation::format::decode_backend_compatibility_fixture(&root.join(POSITIVE_FIXTURE))
        .expect("reviewed planner projection fixture must satisfy Nickel");
    assert!(lattice::operation::format::decode_backend_compatibility_fixture(&root.join(NEGATIVE_FIXTURE)).is_err());
    assert_eq!(lattice::operation::backend::PLANNER_COHORT, PLANNER_REVISION);
}
