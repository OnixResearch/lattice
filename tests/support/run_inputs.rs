//! A parameterized workflow-template bundle on disk with its bound workflow saved in a store.
#![allow(dead_code)]

use std::path::Path;
use std::path::PathBuf;

use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::Workflow;
use lattice::engine::workflow::WorkflowTimestamp;

pub const BUNDLE: &str = "run-inputs-fixture";
pub const PROBE: &str = "probe";
pub const PROBE_COMMAND: &str = "echo \"${LATTICE_PARAM_ENABLED}\" \"${LATTICE_PARAM_COUNT}\" \"${LATTICE_PARAM_FORMAT}\" \"${LATTICE_PARAM_TARGET}\"";
pub const SAVED_FORMAT: &str = "text";
const STORE_FILE: &str = "lattice.redb";

/// Every type: optional `enabled` bool, optional `count` integer 1..=5, required `format` enum,
/// and optional `target` relative path.
pub const PARAMS: &str = r#"[
    { name = "enabled", type = { kind = "bool" }, required = false, default = false,
      bindings = [{ node = "probe", shell_env = "LATTICE_PARAM_ENABLED" }] },
    { name = "count", type = { kind = "integer", min = 1, max = 5 }, required = false, default = 1,
      bindings = [{ node = "probe", shell_env = "LATTICE_PARAM_COUNT" }] },
    { name = "format", type = { kind = "enum", values = ["json", "text"] }, required = true,
      bindings = [{ node = "probe", shell_env = "LATTICE_PARAM_FORMAT" }] },
    { name = "target", type = { kind = "relative_path", max_bytes = 64 }, required = false, default = "out",
      bindings = [{ node = "probe", shell_env = "LATTICE_PARAM_TARGET" }] },
  ]"#;

pub struct Fixture {
    pub dir: tempfile::TempDir,
    pub config: PathBuf,
    pub catalog: PathBuf,
    pub state: PathBuf,
    pub marker: PathBuf,
    /// The bound bundle the fixture saved, for exact re-saves.
    pub saved: lattice::workflow_template::ValidatedBundle,
}

/// The template workflow: a bound probe and an unbound marker whose effect proves a node ran.
pub fn template_workflow(marker: &Path) -> Workflow {
    let mut workflow = Workflow::new(BUNDLE, WorkflowTimestamp::epoch());
    let probe = workflow.add_shell_node(PROBE, PROBE_COMMAND);
    workflow.add_shell_node("marker", format!("touch '{}'", marker.display()));
    let Some(NodeConfig::Shell { env, .. }) = workflow.configs.get_mut(&probe.0) else {
        panic!("probe is a shell node");
    };
    for (key, value) in [("ENABLED", "false"), ("COUNT", "1"), ("TARGET", "out")] {
        env.insert(format!("LATTICE_PARAM_{key}"), value.to_string());
    }
    workflow
}

fn manifest(workflow_pin: &str, params: &str) -> String {
    format!(
        r#"{{
  schema_version = 2,
  id = "{BUNDLE}",
  display_name = "Run inputs fixture",
  description = "Typed run inputs fixture",
  category = "tests",
  mode = "mutating",
  params = {params},
  workflow = {{ path = "workflow.ncl", blake3 = "{workflow_pin}" }},
  assets = [],
  compatibility = {{
    workflow_schema = 1,
    node_kinds = ["shell", "approval"],
    runtime_kinds = ["shell", "approval"],
    feature_ids = ["lattice.workflow-template-bundle.v1"],
  }},
  operator_notes = [],
  non_claims = [],
}}
"#
    )
}

/// Write the bundle with `params` into `catalog`.
pub fn write_bundle(catalog: &Path, workflow: &Workflow, params: &str) {
    let bundle = catalog.join(BUNDLE);
    std::fs::create_dir_all(&bundle).expect("bundle directory");
    let workflow_text = lattice::workflow_format::workflow_to_nickel(workflow);
    let pin = lattice::workflow_template::ContentIdentity::from_bytes(workflow_text.as_bytes());
    std::fs::write(bundle.join("workflow.ncl"), workflow_text).expect("workflow writes");
    std::fs::write(bundle.join("manifest.ncl"), manifest(pin.as_str(), params)).expect("manifest writes");
}

pub fn catalog_template(catalog: &Path) -> lattice::workflow_template::ValidatedBundle {
    let loaded = lattice::workflow_template::load_catalog(catalog).expect("catalog loads");
    assert!(loaded.diagnostics.is_empty(), "diagnostics: {:?}", loaded.diagnostics);
    loaded
        .bundles
        .into_iter()
        .find(|bundle| bundle.manifest().id == BUNDLE)
        .expect("fixture bundle loads")
}

pub fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|item| (*item).to_string()).collect()
}

impl Fixture {
    /// The bundle on disk, a config without effect authority, and the workflow saved from the
    /// bundle bound to `format = "text"`.
    pub fn new() -> Self {
        Self::with_params(PARAMS)
    }

    pub fn with_params(params: &str) -> Self {
        Self::with_workflow(params, template_workflow)
    }

    /// The fixture whose template also holds an approval gate.
    pub fn gated() -> Self {
        Self::with_workflow(PARAMS, |marker| {
            let mut workflow = template_workflow(marker);
            lattice::tui::authoring::node::create_approval(&mut workflow, "gate", "review-gate", "operator")
                .expect("gate adds");
            workflow
        })
    }

    pub fn with_workflow(params: &str, workflow: impl FnOnce(&Path) -> Workflow) -> Self {
        let dir = tempfile::tempdir().expect("fixture directory");
        let catalog = dir.path().join("workflow-templates");
        let state = dir.path().join("state");
        let marker = dir.path().join("marker");
        std::fs::create_dir_all(&state).expect("state directory");
        write_bundle(&catalog, &workflow(&marker), params);
        let template = catalog_template(&catalog);
        let supplied = if template.manifest().params().is_empty() {
            std::collections::BTreeMap::new()
        } else {
            [("format".to_string(), lattice::workflow_template::ParamValue::Text(SAVED_FORMAT.into()))].into()
        };
        let saved = lattice::workflow_template::bind(&template, &supplied).expect("fixture binds");
        let fixture = Self {
            config: dir.path().join("config.ncl"),
            dir,
            catalog,
            state,
            marker,
            saved,
        };
        fixture.write_config(None);
        lattice::workflow_template::save_draft(&fixture.store(), &fixture.saved).expect("bound workflow saves");
        fixture
    }

    pub fn write_config(&self, authority: Option<&Path>) {
        let authority = authority
            .map(|path| format!(" effect_authority = {},", serde_json::to_string(path).expect("path encodes")))
            .unwrap_or_default();
        let state = serde_json::to_string(&self.state).expect("state path encodes");
        let config = format!("{{ data_dir = {state}, default_timeout_secs = 5, shell = \"sh\",{authority} }}\n");
        std::fs::write(&self.config, config).expect("config writes");
    }

    pub fn store_path(&self) -> PathBuf {
        self.state.join(STORE_FILE)
    }

    pub fn store(&self) -> lattice::store::Store {
        lattice::store::Store::new(self.store_path()).expect("store opens")
    }

    /// Read the inputs, then admit them against the fixture catalog.
    pub fn admit_inputs(
        &self,
        store: &lattice::store::Store,
        workflow_name: &str,
        cli: &[String],
        file: Option<&Path>,
    ) -> Result<lattice::workflow_template::inputs::Admitted, lattice::workflow_template::Error> {
        let raw = lattice::workflow_template::InputRequest { cli, file }.read()?;
        lattice::workflow_template::admit_saved_inputs(store, workflow_name, &self.catalog, &raw)
    }

    pub fn input_file(&self, name: &str, json: &str) -> PathBuf {
        let path = self.dir.path().join(name);
        std::fs::write(&path, json).expect("input file writes");
        path
    }

    pub fn cli(&self, arguments: &[&str]) -> std::process::Output {
        lattice(&self.config, arguments)
    }
}

pub fn lattice(config: &Path, arguments: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(config)
        .args(arguments)
        .output()
        .expect("lattice runs")
}
