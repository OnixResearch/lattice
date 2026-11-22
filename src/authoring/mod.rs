//! Captured author inputs and their association with admitted, materialized workflows.
//!
//! A captured-module evaluation observes a native closure; source and graph
//! associations remain distinct from executable authority.

mod captured;
pub mod cli;
pub mod construction;
pub mod shell;

pub use crate::workflow_template::ContentIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProgramKind {
    Workflow,
    /// Source evaluates to { parameters = <record>, build = fun parameters => <workflow> }.
    Parameterized,
    /// Captured graph-construction candidate using the selected Lattice shell target.
    Construction,
    /// Parameterized source whose build result is a graph-construction candidate.
    ParameterizedConstruction,
}

/// Reuses the workflow-template byte/path bounds and content identity type.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
// r[impl lattice.source_authoring.capture]
pub struct CapturedProgram {
    pub entry: String,
    pub files: std::collections::BTreeMap<String, Vec<u8>>,
    pub kind: ProgramKind,
    pub parameters: std::collections::BTreeMap<String, serde_json::Value>,
}

impl CapturedProgram {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.files.is_empty()
            || self.files.len() > crate::workflow_template::model::MAX_BUNDLE_FILES
            || !self.files.contains_key(&self.entry)
        {
            return Err("captured program file set is invalid");
        }
        let mut total = 0usize;
        for (path, bytes) in &self.files {
            if !safe_path(path) || bytes.len() > crate::workflow_template::model::MAX_FILE_BYTES {
                return Err("captured program path or file limit is invalid");
            }
            total = total.checked_add(bytes.len()).ok_or("captured program byte limit exceeded")?;
        }
        let parameter_bytes = serde_json::to_vec(&self.parameters).map_err(|_| "parameters are invalid")?;
        if total > crate::workflow_template::model::MAX_BUNDLE_BYTES
            || parameter_bytes.len() > crate::workflow_template::model::MAX_FILE_BYTES
            || self.parameters.len() > 64
        {
            return Err("captured program byte or parameter limit exceeded");
        }
        Ok(())
    }

    pub fn identity(&self) -> Result<ContentIdentity, &'static str> {
        self.validate()?;
        identity("lattice.author-inputs.v1", self)
    }
}

pub fn safe_path(path: &str) -> bool {
    path.len() <= 512
        && crate::workflow_template::safe_relative_path(path)
        && !path.contains('\\')
        && !path.chars().any(char::is_control)
}

/// Observed compiler inputs, not a claim about registry executable bytes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerIdentity {
    pub evaluator_source: String,
    pub evaluator_library: String,
    pub evaluator_core: String,
    pub dependency_lock: ContentIdentity,
    pub lowerer_adapter: ContentIdentity,
    pub source_adapter: ContentIdentity,
    pub profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_artifact: Option<String>,
}

impl CompilerIdentity {
    pub fn current() -> Result<Self, &'static str> {
        Ok(Self {
            evaluator_source: crate::nickel::SOURCE_REVISION.into(),
            evaluator_library: crate::nickel::LIBRARY_VERSION.into(),
            evaluator_core: crate::nickel::CORE_VERSION.into(),
            dependency_lock: frozen_identity(env!("LATTICE_DEPENDENCY_LOCK_BLAKE3"))?,
            lowerer_adapter: frozen_identity(env!("LATTICE_WORKFLOW_LOWERER_BLAKE3"))?,
            source_adapter: frozen_identity(env!("LATTICE_SOURCE_ADAPTER_BLAKE3"))?,
            worker_artifact: crate::nickel::captured_worker().ok().map(|(_, digest)| digest),
            profile: "lattice.author-association.v2".into(),
        })
    }
}

fn frozen_identity(value: &str) -> Result<ContentIdentity, &'static str> {
    ContentIdentity::try_from_hex(value).map_err(|_| "build-generated compiler identity is invalid")
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AssociationState {
    /// Historical import-free source-text evaluation, without native closure evidence.
    CapturedSnapshot,
    /// Captured-only native evaluation receipt; authenticity requires shell replay.
    CapturedModules,
    /// File-context import retained these entry bytes. The evaluator did not expose
    /// a captured-byte request/closure receipt; no exact-input replay claim is made.
    RetainedEntrySnapshot,
    /// Materialized edits do not reproduce from the retained source revision.
    MaterializedFork,
}

/// Exact native worker request/response, not an attestation of the selected worker.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatorObservation {
    pub modules: std::collections::BTreeMap<String, Vec<u8>>,
    pub entry: String,
    pub result: crate::nickel::Evaluated,
    pub worker_artifact: String,
}

impl EvaluatorObservation {
    pub fn validate(&self) -> Result<(), &'static str> {
        crate::nickel::validate_captured_response(&self.entry, &self.modules, &self.result)
            .map_err(|_| "captured evaluator observation changed")
    }
}

/// Native candidate, projection, binding and bounded origin facts from one compilation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructionAssociation {
    pub projection_json: Vec<u8>,
    pub binding_json: Vec<u8>,
    pub origins: Vec<choregraph_nickel::construction::model::projection::Origin>,
    /// Runtime port ID to native projection port ID; never inferred from labels.
    #[serde(default = "std::collections::BTreeMap::new")]
    pub port_map: std::collections::BTreeMap<u64, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAssociation {
    pub program: std::sync::Arc<CapturedProgram>,
    pub source_revision: ContentIdentity,
    pub compiler: CompilerIdentity,
    pub evaluated_output: ContentIdentity,
    pub expanded_artifact: ContentIdentity,
    pub state: AssociationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observation: Option<EvaluatorObservation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub construction: Option<ConstructionAssociation>,
}

/// Pure structural/receipt correspondence checks, not evidence that a worker ran.
/// Trust-boundary callers must use `shell::verify_observation` or a freshly
/// rebuilt exact source preview before admitting captured provenance.
impl SourceAssociation {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.program.identity()? != self.source_revision {
            return Err("captured source revision identity differs");
        }
        if self.state == AssociationState::RetainedEntrySnapshot
            && (self.observation.is_some() || self.construction.is_some())
        {
            return Err("retained entry snapshot cannot carry captured evaluation or construction evidence");
        }
        if let Some(observation) = &self.observation {
            if self.state == AssociationState::CapturedModules
                && self.evaluated_output != ContentIdentity::from_bytes(observation.result.json.as_bytes())
            {
                return Err("captured output identity changed");
            }
            observation.validate()?;
            if self.state == AssociationState::CapturedModules
                && (self.compiler.worker_artifact.as_deref() != Some(observation.worker_artifact.as_str())
                    || self.program.files.iter().any(|(name, bytes)| observation.modules.get(name) != Some(bytes)))
            {
                return Err("captured source/evaluator association changed");
            }
        } else if self.state == AssociationState::CapturedModules {
            return Err("captured evaluation observation is unavailable");
        }
        Ok(())
    }
}

pub fn identity<T: serde::Serialize>(domain: &str, value: &T) -> Result<ContentIdentity, &'static str> {
    // Sort explicitly even when serde_json's preserve_order feature is enabled.
    let mut value = serde_json::to_value(value).map_err(|_| "identity encoding failed")?;
    value.sort_all_objects();
    let bytes = serde_json::to_vec(&(domain, value)).map_err(|_| "identity encoding failed")?;
    Ok(ContentIdentity::from_bytes(&bytes))
}

pub fn expanded_identity(workflow: &crate::engine::Workflow) -> Result<ContentIdentity, &'static str> {
    // The published v1 identity predates construction facts. Preserve its exact
    // six-field encoding for unchanged legacy workflows, not a nine-field v1.
    if !workflow.control_dependencies.is_empty()
        || workflow.construction_profile
        || workflow.construction_runtime.is_some()
    {
        return construction_identity(workflow);
    }
    identity(
        "lattice.materialized-workflow.v1",
        &(
            &workflow.meta,
            &workflow.graph,
            &workflow.configs,
            &workflow.credentials,
            &workflow.runner,
            &workflow.triggers,
        ),
    )
}

fn construction_identity(workflow: &crate::engine::Workflow) -> Result<ContentIdentity, &'static str> {
    identity(
        "lattice.materialized-workflow.v2",
        &(
            &workflow.meta,
            &workflow.graph,
            &workflow.configs,
            &workflow.credentials,
            &workflow.runner,
            &workflow.triggers,
            &workflow.control_dependencies,
            &workflow.construction_profile,
            &workflow.construction_runtime,
        ),
    )
}
/// Whether the workflow no longer equals the expansion its source association recorded, so
/// reconciliation would mark the association a materialized fork. `None` without an association.
pub fn diverges_from_expansion(workflow: &crate::engine::Workflow) -> Result<Option<bool>, &'static str> {
    let Some(association) = workflow.authoring.as_ref() else {
        return Ok(None);
    };
    association.validate()?;
    Ok(Some(expanded_identity(workflow)? != association.expanded_artifact))
}

/// Shared rule for graph mutation, typed editing, saving and exporting.
/// No inverse compiler or source rewrite is attempted, including after a revert.
// r[impl lattice.source_authoring.fork]
pub fn reconcile_materialized(workflow: &mut crate::engine::Workflow) -> Result<(), &'static str> {
    if diverges_from_expansion(workflow)? == Some(true) {
        workflow.authoring.as_mut().ok_or("source association disappeared")?.state = AssociationState::MaterializedFork;
    }
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDraft {
    pub id: String,
    pub workflow_name: String,
    pub expected_revision: Option<ContentIdentity>,
    pub program: CapturedProgram,
    /// Bounded category only, never evaluator output, paths or source text.
    pub diagnostic: Option<String>,
    pub preview: Option<SourcePreview>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourcePreview {
    pub input_identity: ContentIdentity,
    pub compiler: CompilerIdentity,
    pub candidate: crate::engine::Workflow,
}

impl SourceDraft {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.id.is_empty()
            || self.id.len() > 128
            || !self.id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err("draft identifier is invalid");
        }
        if self.workflow_name.is_empty() || self.workflow_name.len() > 512 {
            return Err("draft workflow name is invalid");
        }
        if self.diagnostic.as_ref().is_some_and(|message| message.len() > 128) {
            return Err("draft diagnostic is too large");
        }
        self.program.validate()
    }
}

/// Never infer missing template, instance or source-span attribution.
pub const ORIGIN_UNAVAILABLE: &str = "unavailable: retained source has no construction origin for this element";
pub const COMPLETE_MODULE_PROVENANCE_UNAVAILABLE: &str =
    "unavailable: workflow has no retained captured-module evaluation receipt";
