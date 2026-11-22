//! Pure, conservative effect classification for workflow-template bundle modes.

/// The declared upper bound on what a bundle's workflow may do to the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleMode {
    /// Every node is pure or provably read-only.
    ReadOnly,
    /// At least one node may change a repository, host, network peer, or agent state.
    Mutating,
}

impl BundleMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::Mutating => "mutating",
        }
    }
}

/// Classification of one node's effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectClass {
    /// No external effect: control flow, merge, or an approval gate.
    Pure,
    /// A bounded local read through an operator-configured root.
    ReadOnly,
    /// Anything not provably read-only.
    Mutating,
}

// r[impl lattice.workflow_templates.mode]
/// Classify one node conservatively. Any effect that is not provably read-only is mutating.
///
/// A shell or process command, an HTTP request of any method, an agent call, a file write or
/// append, and a subworkflow whose child is outside the bundle are all mutating. The match is
/// exhaustive so that a new node kind needs an explicit classification.
pub fn classify_node(config: &crate::engine::workflow::NodeConfig) -> EffectClass {
    use crate::engine::workflow::NodeConfig;
    match config {
        NodeConfig::Condition(_) | NodeConfig::Merge(_) | NodeConfig::Approval(_) => EffectClass::Pure,
        NodeConfig::File { operation, .. } => match operation {
            crate::file_node::FileOperationKind::Read => EffectClass::ReadOnly,
            crate::file_node::FileOperationKind::Write | crate::file_node::FileOperationKind::Append => {
                EffectClass::Mutating
            }
        },
        NodeConfig::ImageInput { .. } => EffectClass::ReadOnly,
        NodeConfig::Shell { .. }
        | NodeConfig::Http { .. }
        | NodeConfig::Agent(_)
        | NodeConfig::Operation(_)
        | NodeConfig::Subworkflow(_) => EffectClass::Mutating,
    }
}

/// The least mode that admits every node of the workflow.
pub fn required_mode(workflow: &crate::engine::Workflow) -> BundleMode {
    let is_mutating = workflow.configs.values().any(|config| classify_node(config) == EffectClass::Mutating);
    if is_mutating {
        BundleMode::Mutating
    } else {
        BundleMode::ReadOnly
    }
}

/// Whether a bundle declared with `declared` may carry this workflow.
pub fn admits(declared: BundleMode, workflow: &crate::engine::Workflow) -> bool {
    declared == BundleMode::Mutating || required_mode(workflow) == BundleMode::ReadOnly
}
