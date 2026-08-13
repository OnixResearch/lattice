//! Static adoption checks for exact Choregraph source and authority boundaries.

const CHOREGRAPH_REVISION: &str = "85ea5e35caaab23260fcb27bc7c5d995dec546c4";
const CHOREGRAPH_ROLLBACK_REVISION: &str = "81b260da2256147ec82cb5dd44c0a0c1dcb7500d";
const CHOREGRAPH_URL: &str = "https://git.onix.computer/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git";
const VCS_REVISION_HEX_LENGTH: usize = 40;
const CARGO_PIN_COUNT: usize = 4;
const CARGO_MANIFEST: &str = include_str!("../Cargo.toml");
const FLAKE_SOURCE: &str = include_str!("../flake.nix");
const SOURCE_ADMISSION: &str = include_str!("fixtures/radicle/choregraph-source-admission-v1.ncl");
const FLOATING_SOURCE: &str = include_str!("fixtures/radicle/negative/choregraph-floating-source.ncl");
const LOWERING_CORE: &str = include_str!("../src/workflow_format/choregraph.rs");
const WORKFLOW_SHELL: &str = include_str!("../src/workflow_format.rs");
const RUNNER_CORE: &str = include_str!("../src/engine/runner.rs");
const EXECUTOR_SHELL: &str = include_str!("../src/engine/executor.rs");
const RUNNER_TESTS: &str = include_str!("test_runner_core.rs");
const MIGRATION_TESTS: &str = include_str!("test_choregraph_migration.rs");
const CHECK_SCRIPT: &str = include_str!("../scripts/check.rs");
const BASALT_SOURCE: &str = include_str!("../vendor-cargo-deps/basalt/SOURCE");
const BASALT_NORMALIZATION: &str = include_str!("../vendor-cargo-deps/basalt/NORMALIZATION.md");
const FORBIDDEN_LOWERING_MARKERS: &[&str] = &[
    "std::fs",
    "std::process",
    "std::net",
    "std::env",
    "read_to_string",
    "read_dir",
    "tokio::",
    "async ",
    "unsafe",
];

// r[verify lattice.choregraph_adoption.prerequisites]
// r[verify lattice.choregraph_adoption.prerequisites.scenario.active]
// r[verify lattice.choregraph_adoption.source]
// r[verify lattice.choregraph_adoption.source.scenario.identity]
// r[verify lattice.choregraph_adoption.source_alignment]
// r[verify lattice.choregraph_adoption.source_alignment.scenario.drift]
#[test]
fn exact_source_and_prerequisites_are_aligned() {
    assert_eq!(CARGO_MANIFEST.matches(CHOREGRAPH_REVISION).count(), CARGO_PIN_COUNT);
    assert!(CARGO_MANIFEST.contains("choregraph-core"));
    assert!(CARGO_MANIFEST.contains("choregraph-nickel"));
    assert!(CARGO_MANIFEST.contains("choregraph-seaglass"));
    assert!(CARGO_MANIFEST.contains("choregraph-kamacite"));
    assert!(CARGO_MANIFEST.contains(CHOREGRAPH_URL));
    assert!(FLAKE_SOURCE.contains(CHOREGRAPH_REVISION));
    assert!(FLAKE_SOURCE.contains(CHOREGRAPH_URL));
    assert!(SOURCE_ADMISSION.contains(CHOREGRAPH_REVISION));
    assert!(SOURCE_ADMISSION.contains("execution_graph_baseline = true"));
    assert!(SOURCE_ADMISSION.contains("nickel_evaluator_baseline = true"));
    assert!(FLOATING_SOURCE.contains("admission.revision | force = \"main\""));
    assert!(FLOATING_SOURCE.contains("admission.source_url | force = \"https://github.com/onixresearch/choregraph\""));
    assert!(FLOATING_SOURCE.contains("admission.evidence.single_step_proof | force = false"));
}

// r[verify lattice.choregraph_adoption.inventory]
// r[verify lattice.choregraph_adoption.inventory.scenario.missing]
// r[verify lattice.choregraph_adoption.lowering.scenario.filesystem]
// r[verify lattice.choregraph_adoption.adapter.scenario.duplicate]
// r[verify lattice.choregraph_adoption.parity]
// r[verify lattice.choregraph_adoption.parity.scenario.mismatch]
// r[verify lattice.choregraph_adoption.single_implementation]
// r[verify lattice.choregraph_adoption.single_implementation.scenario.legacy]
// r[verify lattice.choregraph_adoption.authority]
// r[verify lattice.choregraph_adoption.authority.scenario.effect]
// r[verify lattice.choregraph_adoption.rollback]
// r[verify lattice.choregraph_adoption.rollback.scenario.baseline]
// r[verify lattice.choregraph_adoption.validation]
// r[verify lattice.choregraph_adoption.validation.scenario.persistence]
#[test]
fn adoption_boundaries_and_evidence_paths_remain_present() {
    for marker in FORBIDDEN_LOWERING_MARKERS {
        assert!(!LOWERING_CORE.contains(marker), "forbidden lowering marker: {marker}");
    }
    assert!(WORKFLOW_SHELL.contains("choregraph::lower_portable(&portable)?"));
    assert!(RUNNER_CORE.contains("seaglass_execution_graph::admit_graph"));
    assert!(RUNNER_CORE.contains("migrate_persisted_states"));
    assert!(RUNNER_CORE.contains("replay_persisted_events"));
    assert!(!EXECUTOR_SHELL.contains("choregraph_nickel"));
    assert!(RUNNER_TESTS.contains("shared_graph_core_reports_critical_path_and_bottlenecks"));
    assert!(RUNNER_TESTS.contains("lifecycle_rejects_invalid_and_terminal_transitions"));
    assert!(MIGRATION_TESTS.contains("replay_reconstructs_state_without_intents"));
    assert!(CHECK_SCRIPT.contains("const CLIPPY_ARGS"));
    assert!(CHECK_SCRIPT.contains("name: \"tests\""));
    assert_eq!(CHOREGRAPH_ROLLBACK_REVISION.len(), VCS_REVISION_HEX_LENGTH);
    assert_ne!(CHOREGRAPH_ROLLBACK_REVISION, CHOREGRAPH_REVISION);
    assert!(BASALT_SOURCE.contains("revision: bb8fb5f4a763f532df96b6e65407163ac668834e"));
    assert!(BASALT_SOURCE.contains("ucan-revision: 277ab768eb532ac0c715a3bbff2a7dc37a702f11"));
    assert!(BASALT_NORMALIZATION.contains("not a byte-identical copy"));
    assert!(BASALT_NORMALIZATION.contains("no runtime, sibling, or network fallback"));
}
