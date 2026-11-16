#[path = "support/cairn.rs"]
mod cairn;

use std::fs;

// r[verify lattice.repository_validation.native_lifecycle]
// r[verify lattice.repository_validation.native_lifecycle.scenario.parity]
// r[verify lattice.repository_validation.native_lifecycle.scenario.legacy_rejected]
// r[verify lattice.repository_validation.native_lifecycle.scenario.active_paths]
#[test]
fn reviewed_legacy_migration_preserves_hashes_and_current_admission_reads_only_native_paths() -> cairn::TestResult {
    let fixture = cairn::Fixture::new()?;
    let native = fixture.root.path().join(".cairn");
    if native.exists() {
        fs::remove_dir_all(&native)?;
    }
    let spec = "# Native lifecycle fixture\n\n## Purpose\n\nExercises native lifecycle selection.\n\n## Requirements\n\n### Requirement: Native selection\n\nr[fixture.native_selection]\n\nThe fixture MUST select native lifecycle content.\n\n#### Scenario: Current path\n\nr[fixture.native_selection.current]\n- GIVEN native content exists\n- WHEN status reads the fixture\n- THEN it MUST report the native requirement\n";
    let archived = b"reviewed archive bytes\n";
    fixture.write("cairn/specs/native-selection/spec.md", spec)?;
    fixture.write("cairn/archive/reviewed/receipt.txt", archived)?;
    let rejected = fixture
        .command()
        .args([
            "status",
            "--root",
            ".",
            "--policy",
            concat!(env!("CARGO_MANIFEST_DIR"), "/cairn-policy/generated/cairn-policy.json"),
        ])
        .output()?;
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("cairn.lifecycle_directory.migration_required"));

    let (success, preview) = fixture.run(&["migrate", "cairn"])?;
    assert!(success, "migration preview: {preview}");
    assert_eq!(preview["dry_run"], true);
    assert_eq!(preview["mutated"], false);
    assert!(!native.exists());
    let actions = preview["actions"].as_array().ok_or("migration actions")?;
    assert_eq!(actions.len(), 2);
    for action in actions {
        let source = fixture.root.path().join(action["source"].as_str().ok_or("source")?);
        let bytes = fs::read(source)?;
        assert_eq!(action["source_hash"], blake3::hash(&bytes).to_hex().to_string());
    }
    let plan_hash = preview["plan_hash"].as_str().ok_or("plan hash")?;
    let (success, executed) = fixture.run(&["migrate", "cairn", "--execute", "--plan-hash", plan_hash])?;
    assert!(success, "migration execution: {executed}");
    assert_eq!(executed["plan_hash"], preview["plan_hash"]);
    assert_eq!(executed["mutated"], true);
    for action in actions {
        let destination = fixture.root.path().join(action["destination"].as_str().ok_or("destination")?);
        let bytes = fs::read(destination)?;
        assert_eq!(action["source_hash"], blake3::hash(&bytes).to_hex().to_string());
    }
    assert_eq!(fs::read(native.join("specs/native-selection/spec.md"))?, spec.as_bytes());
    assert_eq!(fs::read(native.join("archive/reviewed/receipt.txt"))?, archived);

    // Leave contradictory legacy content behind: the current reader must not
    // merge or fall back to it after the exact reviewed copy has completed.
    fixture.write("cairn/specs/legacy-only/spec.md", "# Legacy-only content\n")?;
    let (success, status) = fixture.run(&["status"])?;
    assert!(success, "native status: {status}");
    assert_eq!(status["layout"], "cairn");
    let specs = status["specs"].as_array().ok_or("native specs")?;
    assert_eq!(specs.len(), 1);
    assert_eq!(fs::read(native.join("specs/native-selection/spec.md"))?, spec.as_bytes());
    Ok(())
}
