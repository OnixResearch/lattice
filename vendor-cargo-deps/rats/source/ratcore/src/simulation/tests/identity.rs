use super::super::*;

#[test]
fn registry_orders_namespaces_and_resolves_reviewed_aliases() {
    let registry = SimulationIdentityRegistry::new(vec![
        SimulationIdentityEntry::new(SimulationIdentityKind::Probe, "probe.current").with_aliases(["probe.previous"]),
        SimulationIdentityEntry::new(SimulationIdentityKind::Scenario, "scenario.alpha"),
    ])
    .expect("registry is valid");

    assert_eq!(registry.entries()[0].kind, SimulationIdentityKind::Scenario);
    assert_eq!(registry.resolve(SimulationIdentityKind::Probe, "probe.previous"), Some("probe.current"));
    assert!(
        registry.require(SimulationIdentityKind::Probe, "probe.previous").is_err(),
        "emitted evidence requires the current identity"
    );
}

#[test]
fn duplicate_alias_unknown_and_source_location_identities_fail_closed() {
    let issues = SimulationIdentityRegistry::new(vec![
        SimulationIdentityEntry::new(SimulationIdentityKind::Probe, "probe.same"),
        SimulationIdentityEntry::new(SimulationIdentityKind::Probe, "probe.same"),
        SimulationIdentityEntry::new(SimulationIdentityKind::Probe, "src/probe.rs:10")
            .with_aliases(["probe.same", "probe.same"]),
    ])
    .expect_err("malformed registry must fail");
    let codes: Vec<_> = issues.iter().map(|issue| issue.code).collect();

    assert!(codes.contains(&SimulationIdentityDiagnosticCode::DuplicateIdentity));
    assert!(codes.contains(&SimulationIdentityDiagnosticCode::SourceLocationIdentity));
    assert!(codes.contains(&SimulationIdentityDiagnosticCode::ConflictingAlias));
}

#[test]
fn deterministic_sdk_projection_rejects_real_fnv_collision() {
    assert_eq!(
        project_simulation_id("costarring"),
        project_simulation_id("liquid"),
        "known FNV-1a collision exercises the narrow SDK boundary"
    );
    let issues = SimulationIdentityRegistry::new(vec![
        SimulationIdentityEntry::new(SimulationIdentityKind::Probe, "costarring"),
        SimulationIdentityEntry::new(SimulationIdentityKind::Invariant, "liquid"),
    ])
    .expect_err("SDK collision must fail");

    assert!(
        issues
            .iter()
            .any(|issue| { issue.code == SimulationIdentityDiagnosticCode::SdkProjectionCollision })
    );
}

#[test]
fn unknown_current_identity_reports_stable_namespace() {
    let issue = super::registry()
        .require(SimulationIdentityKind::Probe, "simulation.probe.unknown-v1")
        .expect_err("unknown probe must fail");

    assert_eq!(issue.code, SimulationIdentityDiagnosticCode::UnknownIdentity);
    assert_eq!(issue.kind, Some(SimulationIdentityKind::Probe));
}
