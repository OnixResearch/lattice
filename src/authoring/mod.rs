//! Captured author inputs and their association with admitted, materialized workflows.
//!
//! These records are not Choregraph construction receipts or executable identities.
//! The selected evaluator cannot resolve imports exclusively from a supplied byte map.

pub mod admission;
pub mod cli;
pub mod shell;

pub use crate::workflow_template::ContentIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProgramKind {
    Workflow,
    /// Source evaluates to { parameters = <record>, build = fun parameters => <workflow> }.
    Parameterized,
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
            profile: "lattice.captured-entry.snapshot-only.v1".into(),
        })
    }
}

fn frozen_identity(value: &str) -> Result<ContentIdentity, &'static str> {
    ContentIdentity::try_from_hex(value).map_err(|_| "build-generated compiler identity is invalid")
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AssociationState {
    /// Evaluation observed these inputs and this output. Complete import closure is unavailable.
    CapturedSnapshot,
    /// File-context import retained these entry bytes. The evaluator did not expose
    /// a captured-byte request/closure receipt; no exact-input replay claim is made.
    RetainedEntrySnapshot,
    /// Materialized edits do not reproduce from the retained source revision.
    MaterializedFork,
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
}

impl SourceAssociation {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.program.identity()? != self.source_revision {
            return Err("captured source revision identity differs");
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
/// Shared rule for graph mutation, typed editing, saving and exporting.
/// No inverse compiler or source rewrite is attempted, including after a revert.
// r[impl lattice.source_authoring.fork]
pub fn reconcile_materialized(workflow: &mut crate::engine::Workflow) -> Result<(), &'static str> {
    let Some(association) = workflow.authoring.as_ref() else {
        return Ok(());
    };
    association.validate()?;
    if expanded_identity(workflow)? != association.expanded_artifact {
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

/// There is deliberately no inferred template/instance/source-span mapping.
pub const ORIGIN_UNAVAILABLE: &str = "unavailable: selected Choregraph API has no construction/origin map";
pub const COMPLETE_MODULE_PROVENANCE_UNAVAILABLE: &str =
    "unavailable: selected nickel-eval cannot evaluate imports exclusively from captured module bytes";
