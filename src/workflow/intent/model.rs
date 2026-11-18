/// Exact host-normalized source cohort; presentation-only views are not inputs.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceCohort {
    pub producer_identity: String,
    pub selection_identity: String,
    pub obligations_identity: String,
    pub profile_identity: String,
    pub context_identity: String,
    pub source_identity: String,
    pub policy_identity: String,
    pub dependency_identity: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextCoverage {
    Complete,
    Restricted,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationContract {
    pub id: String,
    pub schema: String,
    pub predicate_identity: String,
}

/// Contract names are data, never commands, permissions or acceptance verdicts.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Obligation {
    pub id: String,
    pub obligation_identity: String,
    pub guidance_identity: String,
    pub remaining_condition_identity: String,
    pub operation_class: String,
    pub action_identity: String,
    pub prerequisites: Vec<String>,
    pub required_observations: Vec<ObservationContract>,
    pub precondition_identity: String,
    pub required_capabilities: std::collections::BTreeSet<String>,
}

/// Fixed-topology implementation choices cannot remove required obligations.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Alternative {
    pub id: String,
    pub implementation_identity: String,
    pub obligations: std::collections::BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub schema: String,
    pub cohort: SourceCohort,
    pub coverage: ContextCoverage,
    pub unsupported_fields: Vec<String>,
    pub obligations: Vec<Obligation>,
    pub alternatives: Vec<Alternative>,
    pub selected_alternative: Option<String>,
}

/// Explicit observed compatibility, not grants or proof of provider readiness.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerFacts {
    pub expected_cohort: SourceCohort,
    pub compiler_artifact_identity: String,
    pub runtime_profile_identity: String,
    pub available_capabilities: std::collections::BTreeSet<String>,
    pub available_implementations: std::collections::BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompileRequest {
    pub intent: Definition,
    pub facts: CompilerFacts,
}

/// Serialized output is a plan only. It has no conversion to an execution grant.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CompiledPlan {
    pub(super) schema: &'static str,
    pub(super) compiler_revision: &'static str,
    pub(super) identity: String,
    pub(super) graph_identity: choregraph_kamacite::identity::ArtifactIdentity,
    pub(super) graph: choregraph_kamacite::concrete::record::ConcreteMachineDescriptorV2,
    pub(super) intent: Definition,
    pub(super) facts: CompilerFacts,
    pub(super) nodes: Vec<String>,
    pub(super) disposition: &'static str,
}

impl CompiledPlan {
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn intent(&self) -> &Definition {
        &self.intent
    }
    pub fn nodes(&self) -> &[String] {
        &self.nodes
    }
    pub fn graph(&self) -> &choregraph_kamacite::concrete::record::ConcreteMachineDescriptorV2 {
        &self.graph
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ProposalComparison {
    pub base_plan_identity: String,
    pub proposed_plan_identity: String,
    pub added_obligations: Vec<String>,
    pub removed_obligations: Vec<String>,
    pub changed_obligations: Vec<String>,
    pub disposition: &'static str,
}
