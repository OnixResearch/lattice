//! A run with inputs re-verifies its bound content before it reserves a run id.

const REVIEW: &str = "agent-evaluation-catalog-review";
const PARAM: &str = "catalog_root";
const KEY: &str = "LATTICE_PARAM_CATALOG_ROOT";

fn template() -> crate::workflow_template::ValidatedBundle {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("builtins/workflow-templates");
    let catalog = crate::workflow_template::load_catalog(&root).expect("built-in catalog loads");
    catalog
        .bundles
        .into_iter()
        .find(|bundle| bundle.manifest().id == REVIEW)
        .expect("catalog review loads")
}

fn admitted(
    template: &crate::workflow_template::ValidatedBundle,
    root: &str,
) -> crate::workflow_template::inputs::Admitted {
    let saved = crate::workflow_template::bind(template, &std::collections::BTreeMap::new()).expect("defaults bind");
    let binding = crate::workflow_template::inputs::retained_binding(&saved).expect("bound bundle is retained");
    let raw = [(PARAM.to_owned(), crate::workflow_template::inputs::RawInput::Text(root.to_owned()))].into();
    let facts = crate::workflow_template::inputs::SavedFacts {
        workflow_name: REVIEW,
        saved: Some(saved.workflow()),
        binding: Some(&binding),
        template: Some(template),
    };
    crate::workflow_template::inputs::admit(facts, &raw).expect("inputs admit")
}

fn refused(store: &crate::store::Store, admitted: crate::workflow_template::inputs::Admitted) -> String {
    let provider = crate::engine::effect::execution::UnavailableRunContextSource;
    let runtime = tokio::runtime::Builder::new_current_thread().build().expect("runtime builds");
    match runtime.block_on(admitted.run_with_authority_provider(store, &provider)) {
        Err(super::super::super::HeadlessRunError::Validation(diagnostics)) => diagnostics[0].message.clone(),
        other => panic!("drifted bound content must be refused before the run, got {other:?}"),
    }
}

// r[verify lattice.workflow_templates.run_inputs.start]
// r[verify lattice.workflow_templates.run_inputs.start.scenario.drift]
#[test]
fn drifted_bound_content_never_reserves_a_run() {
    let directory = tempfile::tempdir().expect("store directory");
    let store = crate::store::Store::new(directory.path().join("drift.redb")).expect("store opens");
    let template = template();
    let intact = admitted(&template, "fixtures/catalog");
    assert!(crate::workflow_template::inputs::reverify_admitted(&intact).is_ok());

    let mut tampered = intact.clone();
    let workflow = &mut tampered.bundle_mut().workflow;
    let node = workflow.graph.nodes().next().expect("node").id;
    if let Some(crate::engine::workflow::NodeConfig::Shell { env, .. }) = workflow.configs.get_mut(&node.0) {
        env.insert(KEY.to_owned(), "fixtures/other".to_owned());
    }
    assert!(refused(&store, tampered).starts_with("bound_content_drift"), "tampered content is refused");

    let mut swapped = intact;
    *swapped.bundle_mut() = admitted(&template, "fixtures/other").bundle().clone();
    assert!(refused(&store, swapped).starts_with("bound_content_drift"), "a mismatched identity is refused");

    assert!(store.list_runs(4).expect("runs list").is_empty(), "no run record");
    assert!(store.load_run_inputs(crate::engine::RunId(0)).expect("inputs read").is_none(), "no input record");
    assert_eq!(store.next_run_id().expect("run id"), crate::engine::RunId(0), "no run id was reserved");
}
