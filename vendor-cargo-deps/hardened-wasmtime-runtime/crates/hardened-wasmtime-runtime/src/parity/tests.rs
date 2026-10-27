//! Fixtures for the strategy-parity rail: corpus agreement, forced
//! divergence, independence gate, and oracle classification. All items are
//! referenced by fully qualified paths per the Octet import policy.

fn forced_divergence_verdict(case: &super::CorpusCase) -> super::HarnessVerdict {
    let admitted = super::admit_case(case.label).expect("case");
    let left = super::admit_observation(
        &admitted,
        crate::Strategy::Cranelift,
        &crate::NormalizedOutcome::Completed,
    )
    .expect("left observation");
    let right = super::admit_observation(
        &admitted,
        crate::Strategy::Winch,
        &crate::NormalizedOutcome::FuelExhausted,
    )
    .expect("right observation");
    super::compare_outcomes(&admitted, &left, &right)
}

#[test]
// r[verify hardened_wasmtime.strategy_parity]
fn corpus_agrees_across_strategies_with_stable_identities() {
    for case in super::corpus() {
        let first = super::run_differential(&case).expect("first differential");
        let second = super::run_differential(&case).expect("replay differential");

        assert!(matches!(
            first.verdict,
            differential_execution_core::ComparisonVerdict::Agreement
        ));
        assert!(first.mismatch.is_none(), "{}", case.label);
        assert_eq!(first.case_identity_hex.len(), 64);
        assert_eq!(first.case_identity_hex, second.case_identity_hex);
    }

    let cases = super::corpus();
    let arithmetic = super::run_differential(&cases[0]).expect("arithmetic case");
    let trap = super::run_differential(&cases[1]).expect("trap case");
    assert_ne!(arithmetic.case_identity_hex, trap.case_identity_hex);
}

#[test]
// r[verify hardened_wasmtime.strategy_parity]
fn forced_divergence_is_detected_and_named() {
    let verdict = forced_divergence_verdict(&super::corpus()[0]);

    assert!(matches!(
        verdict.verdict,
        differential_execution_core::ComparisonVerdict::Divergence
    ));
    let mismatch = verdict.mismatch.expect("divergence must name a surface");
    assert!(
        matches!(
            mismatch.surface,
            differential_execution_core::MismatchSurface::ValueSchema
                | differential_execution_core::MismatchSurface::ValueBytes
        ),
        "unexpected surface {mismatch:?}"
    );
}

#[test]
// r[verify hardened_wasmtime.independence]
fn independence_gate_rejects_duplicate_and_single_backend() {
    let case = &super::corpus()[0];
    let admitted = super::admit_case(case.label).expect("case");
    let outcome = crate::NormalizedOutcome::Completed;
    let cranelift_first =
        super::admit_observation(&admitted, crate::Strategy::Cranelift, &outcome).expect("first");
    let cranelift_again = super::admit_observation(&admitted, crate::Strategy::Cranelift, &outcome)
        .expect("duplicate");

    let duplicate_error = differential_execution_core::admit_independent_backends(
        Some(&cranelift_first),
        Some(&cranelift_again),
    )
    .expect_err("duplicate implementation must fail");
    assert!(matches!(
        duplicate_error,
        differential_execution_core::IndependenceError::DuplicateImplementation
    ));

    let single_error =
        differential_execution_core::admit_independent_backends(Some(&cranelift_first), None)
            .expect_err("single backend must fail");
    assert!(matches!(
        single_error,
        differential_execution_core::IndependenceError::OneBackend
    ));
}

#[test]
// r[verify hardened_wasmtime.oracle]
fn oracle_classifies_exact_matches_and_drift() {
    let case = &super::corpus()[0];
    let admitted = super::admit_case(case.label).expect("case");
    let completed = crate::NormalizedOutcome::Completed;
    let exhausted = crate::NormalizedOutcome::FuelExhausted;

    let fixture = differential_execution_core::OracleFixture {
        case: admitted.identity(),
        normalization: admitted.profiles().normalization,
        expected: super::admit_observation(&admitted, crate::Strategy::Cranelift, &completed)
            .expect("fixture"),
    };

    let exact = differential_execution_core::classify_with_oracle(
        &super::admit_observation(&admitted, crate::Strategy::Cranelift, &completed).expect("left"),
        &super::admit_observation(&admitted, crate::Strategy::Winch, &completed).expect("right"),
        &fixture,
    )
    .expect("exact classification");
    assert!(matches!(
        exact.left,
        differential_execution_core::OracleClassification::ExactMatch
    ));
    assert!(matches!(
        exact.right,
        differential_execution_core::OracleClassification::ExactMatch
    ));

    let drifted = differential_execution_core::classify_with_oracle(
        &super::admit_observation(&admitted, crate::Strategy::Cranelift, &completed).expect("left"),
        &super::admit_observation(&admitted, crate::Strategy::Winch, &exhausted).expect("right"),
        &fixture,
    )
    .expect("drifted classification");
    assert!(matches!(
        drifted.left,
        differential_execution_core::OracleClassification::ExactMatch
    ));
    assert!(matches!(
        drifted.right,
        differential_execution_core::OracleClassification::Mismatch
    ));
}
