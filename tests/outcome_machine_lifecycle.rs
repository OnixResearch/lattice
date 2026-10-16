//! Exercises Lattice's selected lifecycle policy through the actual Cairn CLI.

use serde_json::Value;
use serde_json::json;

#[path = "support/cairn.rs"]
mod cairn;
use cairn::Fixture;
use cairn::TestResult;

const CHANGE: &str = "consumer-lifecycle";
const CHANGE_ROOT: &str = ".cairn/changes/consumer-lifecycle";

fn observed_coverage() -> TestResult<Vec<u8>> {
    let fresh = Fixture::new()?;
    let id = "fixture.lifecycle.covered";
    fresh.write(".cairn/specs/probe/spec.md", format!("r[{id}] The fixture MUST retain exact linkage.\n"))?;
    fresh.write("src/lib.rs", format!("// r[{} {id}]\n", "impl"))?;
    let (success, first) = fresh.run(&["traceability", "coverage", "--json"])?;
    assert!(success, "{first}");
    let (success, replay) = fresh.run(&["traceability", "coverage", "--json"])?;
    assert!(success, "{replay}");
    assert_eq!(first["receipt_hash"], replay["receipt_hash"]);
    assert_eq!(first["valid"], true);
    Ok(serde_json::to_vec(&first)?)
}

fn delivery_review() -> Value {
    json!({
        "schema": "cairn.outcome-machine-review.v1",
        "route": {
            "kind": "delivery",
            "details": {
                "immediate_outcome": "Lattice lifecycle admission rejects unobserved completion.",
                "current_consumer": "Lattice maintainers",
                "durable_capability_contribution": "Repeatable consumer-policy lifecycle admission.",
                "repeatability_plan": "Run the same selected policy against isolated fixture state."
            }
        },
        "maintenance_owner": "Lattice maintainers",
        "tracked_inputs": ["cairn-policy/generated/cairn-policy.json"],
        "deterministic_checks": ["cairn traceability coverage"],
        "ai_assistance": {
            "used": false,
            "hidden_session_state_required": false,
            "completion_authority": "repository"
        },
        "fresh_state_reproduction": {
            "status": "planned",
            "evidence": [],
            "rationale": "Evaluate coverage from isolated repository-owned fixture state."
        },
        "completion": { "status": "planned", "evidence": [] },
        "activity_counts_are_completion_evidence": false,
        "non_claims": ["This local fixture does not prove Lattice release eligibility."]
    })
}

// r[verify lattice.lifecycle.outcome_machine_profile]
// r[verify lattice.lifecycle.outcome_machine_profile.scenario.planned_sync_rejected]
// r[verify lattice.lifecycle.outcome_machine_profile.scenario.repository_evidence_admitted]
#[test]
fn selected_lifecycle_rejects_planned_and_stale_evidence_then_admits_observed_bindings() -> TestResult {
    let fixture = Fixture::new()?;
    let (success, init) = fixture.run(&["init"])?;
    assert!(success, "{init}");
    let (success, created) = fixture.run(&["change", "create", CHANGE, "--profile", "outcome-machine"])?;
    assert!(success, "{created}");
    assert_eq!(created["workflow_profile"], "outcome-machine");
    fixture.write(
        &format!("{CHANGE_ROOT}/proposal.md"),
        "## Why\n\nLattice needs repeatable lifecycle admission.\n\n## What Changes\n\n- Exercise exact consumer policy.\n\n## Impact\n\n- Local fixture only.\n",
    )?;
    fixture.write(
        &format!("{CHANGE_ROOT}/design.md"),
        "# Design\n\n## Context\n\nLifecycle admission needs tracked inputs.\n\n## Decisions\n\n### Decision: Preserve repository authority\n\n**Choice:** Bind observed local evidence.\n\n**Rationale:** Planned work is not completion.\n",
    )?;
    let id = "fixture.lifecycle.consumer";
    fixture.write(
        &format!("{CHANGE_ROOT}/specs/consumer/spec.md"),
        format!("# Consumer fixture\n\n## ADDED Requirements\n\n### Requirement: Exact consumer evidence\n\nr[{id}] Lattice MUST retain repository authority.\n\n#### Scenario: Evidence is observed\n\n- GIVEN tracked inputs\n- WHEN admission runs\n- THEN evidence MUST remain bound.\n"),
    )?;
    fixture.write(
        &format!("{CHANGE_ROOT}/tasks.md"),
        format!("# Tasks\n\n- [x] [serial] Implement bounded consumer behavior. r[{id}]\n- [x] [serial] Verify observed and rejected evidence. r[{id}]\n"),
    )?;
    let mut review = delivery_review();
    fixture.write(&format!("{CHANGE_ROOT}/outcome-machine.json"), serde_json::to_vec(&review)?)?;
    for stage in ["proposal", "design", "tasks"] {
        let (_, gate) = fixture.run(&["gate", stage, CHANGE])?;
        assert_eq!(gate["verdict"], "PASS", "{gate}");
    }
    let (_, planned) = fixture.run(&["sync", CHANGE])?;
    assert_eq!(planned["blocked"], true, "{planned}");

    let completion = observed_coverage()?;
    let fresh_state = observed_coverage()?;
    fixture.write(&format!("{CHANGE_ROOT}/evidence/completion.json"), &completion)?;
    fixture.write(&format!("{CHANGE_ROOT}/evidence/fresh-state.json"), &fresh_state)?;
    review["completion"] = json!({
        "status": "observed",
        "evidence": [{ "kind": "test", "path": "evidence/completion.json", "blake3": blake3::hash(&completion).to_hex().to_string() }]
    });
    review["fresh_state_reproduction"] = json!({
        "status": "observed",
        "rationale": "",
        "evidence": [{ "kind": "replay", "path": "evidence/fresh-state.json", "blake3": blake3::hash(&fresh_state).to_hex().to_string() }]
    });
    fixture.write(&format!("{CHANGE_ROOT}/outcome-machine.json"), serde_json::to_vec(&review)?)?;
    let (_, observed) = fixture.run(&["sync", CHANGE])?;
    assert_eq!(observed["blocked"], false, "{observed}");
    let (_, status) = fixture.run(&["workflow", "status", CHANGE])?;
    assert_eq!(status["valid"], true, "{status}");

    fixture.write(&format!("{CHANGE_ROOT}/evidence/completion.json"), b"changed after observation\n")?;
    let (_, stale) = fixture.run(&["sync", CHANGE])?;
    assert_eq!(stale["blocked"], true, "{stale}");
    fixture.write(&format!("{CHANGE_ROOT}/evidence/completion.json"), &completion)?;
    fixture.write(&format!("{CHANGE_ROOT}/evidence/fresh-state.json"), b"changed fresh state\n")?;
    let (_, stale_fresh) = fixture.run(&["sync", CHANGE])?;
    assert_eq!(stale_fresh["blocked"], true, "{stale_fresh}");
    fixture.write(&format!("{CHANGE_ROOT}/evidence/fresh-state.json"), &fresh_state)?;
    let (_, synced) = fixture.run(&["sync", CHANGE, "--execute"])?;
    assert_eq!(synced["blocked"], false, "{synced}");
    assert_eq!(synced["mutated"], true, "{synced}");
    Ok(())
}
