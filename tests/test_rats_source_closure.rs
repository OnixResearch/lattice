//! Maintained evidence for the pinned Rats source closure and focused Lattice consumer compile
//! rail.

use std::path::Path;

use pretty_assertions::assert_eq;

const EXPECTED_RATS_REVISION: &str = "5b932c689431cedb05428efe727889455ebe0beb";
const RATS_ROOT: &str = "vendor-cargo-deps/rats/source";
const SOURCE_METADATA: &str = "vendor-cargo-deps/rats/source/SOURCE";
const ASSERTIONS_MODULE: &str = "vendor-cargo-deps/rats/source/ratcore/src/manual/doc/assertions.rs";
const MANUAL_TESTS_MODULE: &str = "vendor-cargo-deps/rats/source/ratcore/src/manual/tests.rs";
const SYNC_SCRIPT: &str = "scripts/sync_rats.rs";
const CHECK_SCRIPT: &str = "scripts/check.rs";
const FLAKE_PATH: &str = "flake.nix";
const EXPECTED_SNAPSHOT_COUNT: usize = 8;
const SNAPSHOT_PATHS: &[&str] = &[
    "docs/generated/snapshots/dialog-basic.subwayrat.typ",
    "docs/generated/snapshots/dialog-basic.meteorite.html",
    "docs/generated/snapshots/chrome-shell.subwayrat.typ",
    "docs/generated/snapshots/chrome-shell.meteorite.html",
    "docs/generated/snapshots/knowledge-workflow.subwayrat.typ",
    "docs/generated/snapshots/knowledge-workflow.meteorite.html",
    "docs/generated/snapshots/theme-tokens.subwayrat.typ",
    "docs/generated/snapshots/theme-tokens.meteorite.html",
];

#[derive(Debug, Clone)]
struct SourceClosure<'a> {
    metadata: &'a str,
    sync_script: &'a str,
    check_script: &'a str,
    flake: &'a str,
    manual_tests: &'a str,
    assertions: &'a str,
    snapshots: Vec<(&'a str, &'a [u8])>,
}

fn validate_source_closure(closure: &SourceClosure<'_>) -> Result<(), String> {
    let revision_line = format!("revision: {EXPECTED_RATS_REVISION}");
    let sync_pin = format!("const EXPECTED_REVISION: &str = \"{EXPECTED_RATS_REVISION}\";");
    if !closure.metadata.lines().any(|line| line == revision_line)
        || !closure.sync_script.contains(&sync_pin)
        || !closure.sync_script.contains("verify_source_head(&source_root)?;")
        || !closure.sync_script.contains("[\"rev-parse\", \"HEAD\"]")
    {
        return Err("Rats revision identity or source-HEAD admission drifted".to_string());
    }
    if !closure.metadata.contains("- ratcore/src") || !closure.metadata.contains("- docs/generated/snapshots") {
        return Err("Rats source metadata omitted required roots".to_string());
    }
    if !closure.manual_tests.contains("pub(super) mod assertions;") {
        return Err("Rats manual tests no longer reference the assertions module".to_string());
    }
    if !closure.check_script.contains(
        "const FOCUSED_CONSUMER_CHECK_ARGS: &[&str] = &[\"check\", \"--package\", \"lattice\", \"--all-targets\"];",
    ) {
        return Err("focused Lattice consumer compile rail is missing".to_string());
    }
    let exact_source_asset = "${sourceRoot}/vendor-cargo-deps/rats/source/SOURCE";
    let exact_snapshot_root = "${sourceRoot}/vendor-cargo-deps/rats/source/docs/generated/snapshots/";
    let broad_source_root = "hasPrefix \"${sourceRoot}/vendor-cargo-deps/rats/source/\" (toString path)";
    if !closure.flake.contains(exact_source_asset)
        || !closure.flake.contains(exact_snapshot_root)
        || !closure.flake.contains("isRatsCompileTimeAssetPath path")
        || closure.flake.contains(broad_source_root)
    {
        return Err("Rats Nix source closure is missing or over-broad".to_string());
    }
    if closure.snapshots.len() != EXPECTED_SNAPSHOT_COUNT {
        return Err("Rats generated snapshot count drifted".to_string());
    }
    for (relative_path, bytes) in &closure.snapshots {
        if bytes.is_empty() {
            return Err(format!("Rats snapshot is empty: {relative_path}"));
        }
        if !closure.assertions.contains(relative_path) {
            return Err(format!("Rats assertions module omitted snapshot: {relative_path}"));
        }
    }
    Ok(())
}

fn repository_closure<'a>(
    root: &'a Path,
    metadata: &'a str,
    sync_script: &'a str,
    check_script: &'a str,
    flake: &'a str,
    manual_tests: &'a str,
    assertions: &'a str,
    snapshot_bytes: &'a [Vec<u8>],
) -> SourceClosure<'a> {
    let snapshots = SNAPSHOT_PATHS.iter().zip(snapshot_bytes).map(|(path, bytes)| (*path, bytes.as_slice())).collect();
    assert!(root.ends_with(RATS_ROOT));
    SourceClosure {
        metadata,
        sync_script,
        check_script,
        flake,
        manual_tests,
        assertions,
        snapshots,
    }
}

// r[verify n8nclone.workflow_execution.rats_dependency]
// r[verify n8nclone.workflow_execution.rats_dependency.scenario.source-completeness]
#[test]
fn pinned_rats_snapshot_is_source_complete_and_has_a_focused_consumer_compile_rail() {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let rats_root = repository_root.join(RATS_ROOT);
    let metadata = std::fs::read_to_string(repository_root.join(SOURCE_METADATA)).expect("Rats SOURCE metadata");
    let sync_script = std::fs::read_to_string(repository_root.join(SYNC_SCRIPT)).expect("Rats sync script");
    let check_script = std::fs::read_to_string(repository_root.join(CHECK_SCRIPT)).expect("repository check script");
    let flake = std::fs::read_to_string(repository_root.join(FLAKE_PATH)).expect("Nix flake");
    let manual_tests = std::fs::read_to_string(repository_root.join(MANUAL_TESTS_MODULE)).expect("Rats manual tests");
    let assertions = std::fs::read_to_string(repository_root.join(ASSERTIONS_MODULE)).expect("Rats assertions module");
    let snapshot_bytes = SNAPSHOT_PATHS
        .iter()
        .map(|path| std::fs::read(rats_root.join(path)).expect("generated Rats snapshot"))
        .collect::<Vec<_>>();
    let closure = repository_closure(
        &rats_root,
        &metadata,
        &sync_script,
        &check_script,
        &flake,
        &manual_tests,
        &assertions,
        &snapshot_bytes,
    );

    validate_source_closure(&closure).expect("complete pinned Rats source closure");
    assert_eq!(closure.snapshots.len(), EXPECTED_SNAPSHOT_COUNT);
}

#[test]
fn rats_source_closure_rejects_revision_missing_module_snapshot_and_compile_drift() {
    let complete_metadata =
        format!("name: rats\nrevision: {EXPECTED_RATS_REVISION}\npaths:\n- ratcore/src\n- docs/generated/snapshots\n");
    let complete_sync = format!(
        "const EXPECTED_REVISION: &str = \"{EXPECTED_RATS_REVISION}\"; verify_source_head(&source_root)?; [\"rev-parse\", \"HEAD\"]"
    );
    let complete_check =
        "const FOCUSED_CONSUMER_CHECK_ARGS: &[&str] = &[\"check\", \"--package\", \"lattice\", \"--all-targets\"];";
    let complete_flake = "${sourceRoot}/vendor-cargo-deps/rats/source/SOURCE ${sourceRoot}/vendor-cargo-deps/rats/source/docs/generated/snapshots/ isRatsCompileTimeAssetPath path";
    let complete_manual_tests = "pub(super) mod assertions;";
    let complete_assertions = SNAPSHOT_PATHS.join("\n");
    let snapshot_storage = SNAPSHOT_PATHS.iter().map(|_| vec![1_u8]).collect::<Vec<_>>();
    let snapshots = SNAPSHOT_PATHS
        .iter()
        .zip(&snapshot_storage)
        .map(|(path, bytes)| (*path, bytes.as_slice()))
        .collect::<Vec<_>>();
    let complete = SourceClosure {
        metadata: &complete_metadata,
        sync_script: &complete_sync,
        check_script: complete_check,
        flake: complete_flake,
        manual_tests: complete_manual_tests,
        assertions: &complete_assertions,
        snapshots: snapshots.clone(),
    };
    validate_source_closure(&complete).expect("complete fixture");

    let wrong_revision = SourceClosure {
        metadata: "revision: wrong\n- ratcore/src\n- docs/generated/snapshots",
        ..complete.clone()
    };
    assert!(validate_source_closure(&wrong_revision).is_err());
    let missing_module = SourceClosure {
        manual_tests: "mod other;",
        ..complete.clone()
    };
    assert!(validate_source_closure(&missing_module).is_err());
    let missing_snapshot = SourceClosure {
        snapshots: snapshots[..snapshots.len() - 1].to_vec(),
        ..complete.clone()
    };
    assert!(validate_source_closure(&missing_snapshot).is_err());
    let missing_compile = SourceClosure {
        check_script: "const OTHER_CHECK: &str = \"cargo check\";",
        ..complete.clone()
    };
    assert!(validate_source_closure(&missing_compile).is_err());
    let broad_flake = SourceClosure {
        flake: "${sourceRoot}/vendor-cargo-deps/rats/source/SOURCE ${sourceRoot}/vendor-cargo-deps/rats/source/docs/generated/snapshots/ isRatsCompileTimeAssetPath path hasPrefix \"${sourceRoot}/vendor-cargo-deps/rats/source/\" (toString path)",
        ..complete
    };
    assert!(validate_source_closure(&broad_flake).is_err());
}
