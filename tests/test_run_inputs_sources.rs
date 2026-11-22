//! Input sources and the bound bundle's directory: file bounds, FIFOs, and large catalogs.

#[path = "support/run_inputs.rs"]
mod fixture;

use fixture::Fixture;
use fixture::args;

const MAX_INPUT_FILE_BYTES: usize = 65_536;

fn padded(length: usize) -> String {
    let object = r#"{"format": "json"}"#;
    format!("{object}{}", " ".repeat(length - object.len()))
}

fn read(cli: &[String], file: &std::path::Path) -> Result<(), lattice::workflow_template::Error> {
    lattice::workflow_template::InputRequest { cli, file: Some(file) }.read().map(|_| ())
}

#[test]
fn the_input_file_is_capped_at_exactly_64_kib() {
    let fixture = Fixture::new();
    let cli = Vec::new();
    let exact = fixture.input_file("exact.json", &padded(MAX_INPUT_FILE_BYTES));
    let store = fixture.store();
    fixture.admit_inputs(&store, fixture::BUNDLE, &cli, Some(&exact)).expect("64 KiB is read");
    let over = fixture.input_file("over.json", &padded(MAX_INPUT_FILE_BYTES + 1));
    assert!(matches!(read(&cli, &over), Err(lattice::workflow_template::Error::InputFile)));
    let missing = fixture.dir.path().join("missing.json");
    for path in [missing.as_path(), fixture.dir.path()] {
        assert!(matches!(read(&cli, path), Err(lattice::workflow_template::Error::InputFile)), "{}", path.display());
    }
}

#[cfg(unix)]
#[test]
fn a_fifo_input_file_is_refused_without_blocking() {
    let fixture = Fixture::new();
    let fifo = fixture.dir.path().join("inputs.fifo");
    let made = std::process::Command::new("mkfifo").arg(&fifo).status().expect("mkfifo runs");
    assert!(made.success(), "mkfifo creates a FIFO");
    let (sender, receiver) = std::sync::mpsc::channel();
    let path = fifo.clone();
    std::thread::spawn(move || {
        let refused = read(&[], &path);
        sender
            .send(matches!(refused, Err(lattice::workflow_template::Error::InputFile)))
            .expect("result sends");
    });
    let refused = receiver.recv_timeout(std::time::Duration::from_secs(5)).expect("a FIFO never blocks the read");
    assert!(refused, "a FIFO is not a regular file");

    let output = fixture.cli(&[
        "run",
        fixture::BUNDLE,
        "--input-file",
        fifo.to_str().expect("utf-8 path"),
    ]);
    assert!(!output.status.success());
    assert!(fixture.store().list_runs(4).expect("runs list").is_empty(), "the store stayed usable and untouched");
}

#[test]
fn a_catalog_over_its_directory_limit_still_binds_the_retained_bundle() {
    let fixture = Fixture::new();
    for index in 0..lattice::workflow_template::MAX_CATALOG_BUNDLES {
        std::fs::create_dir(fixture.catalog.join(format!("unrelated-{index:02}"))).expect("directory");
    }
    let listed = lattice::workflow_template::load_catalog(&fixture.catalog).expect("catalog reads");
    assert!(listed.bundles.is_empty(), "the whole catalog is over its limit");
    let store = fixture.store();
    let admitted = fixture.admit_inputs(&store, fixture::BUNDLE, &args(&["format=json"]), None);
    assert!(admitted.is_ok(), "only the retained bundle directory is loaded: {admitted:?}");
}

fn refusal(fixture: &Fixture) -> lattice::workflow_template::inputs::InputError {
    let store = fixture.store();
    match fixture.admit_inputs(&store, fixture::BUNDLE, &args(&["format=json"]), None) {
        Err(lattice::workflow_template::Error::Inputs(error)) => error,
        other => panic!("expected an input refusal, got {other:?}"),
    }
}

// r[verify lattice.workflow_templates.run_inputs.retained]
#[test]
fn an_edited_or_replaced_valid_bundle_is_unavailable() {
    let edited = Fixture::new();
    let mut workflow = fixture::template_workflow(&edited.marker);
    workflow.add_shell_node("extra", "true");
    fixture::write_bundle(&edited.catalog, &workflow, fixture::PARAMS);
    fixture::catalog_template(&edited.catalog);
    assert_eq!(refusal(&edited), lattice::workflow_template::inputs::InputError::TemplateUnavailable);

    let replaced = Fixture::new();
    let mut other =
        lattice::engine::Workflow::new(fixture::BUNDLE, lattice::engine::workflow::WorkflowTimestamp::epoch());
    other.add_shell_node("other", "true");
    fixture::write_bundle(&replaced.catalog, &other, "[]");
    fixture::catalog_template(&replaced.catalog);
    assert_eq!(refusal(&replaced), lattice::workflow_template::inputs::InputError::TemplateUnavailable);
}

#[cfg(unix)]
#[test]
fn a_symlinked_bundle_directory_is_refused_like_the_catalog() {
    let fixture = Fixture::new();
    let bundle = fixture.catalog.join(fixture::BUNDLE);
    let moved = fixture.dir.path().join("elsewhere");
    std::fs::rename(&bundle, &moved).expect("bundle moves");
    std::os::unix::fs::symlink(&moved, &bundle).expect("symlink");
    let listed = lattice::workflow_template::load_catalog(&fixture.catalog).expect("catalog reads");
    assert!(listed.bundles.is_empty(), "the catalog loader skips a symlinked bundle");
    let refused = refusal(&fixture);
    let lattice::workflow_template::inputs::InputError::TemplateLoad(diagnostic) = refused else {
        panic!("a symlinked bundle is refused with its diagnostic, got {refused:?}");
    };
    assert_eq!(diagnostic.code, lattice::workflow_template::DiagnosticCode::SymlinkRejected);
}

#[test]
fn saved_workflow_drift_is_reported_before_a_bundle_load_failure() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let mut edited = store.load_workflow(fixture::BUNDLE).unwrap().unwrap();
    let marker = edited.graph.nodes().find(|node| node.label == "marker").expect("marker").id;
    edited.configs.insert(marker.0, lattice::engine::workflow::NodeConfig::shell("true"));
    store.save_workflow(&edited).expect("operator edit saves");
    drop(store);
    std::fs::write(fixture.catalog.join(fixture::BUNDLE).join("manifest.ncl"), "{}").expect("bundle breaks");
    assert_eq!(refusal(&fixture), lattice::workflow_template::inputs::InputError::SavedWorkflowDrift);
}
