#![allow(non_trait_imports)]

mod engine;

use alloc::string::String;
use alloc::vec::Vec;

pub use engine::{materialize_execution_plan, materialize_execution_plan_with_rotation};

/// Materialized router execution plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterExecutionPlan {
    pub(super) recipe_id: String,
    pub(super) family: crate::RouteFamily,
    pub(super) selected_models: Vec<String>,
    pub(super) output_contract: crate::OutputContract,
    pub(super) bounds: crate::Bounds,
    pub(super) privacy: crate::PrivacyBand,
    pub(super) fallback_eligibility: crate::FallbackEligibility,
    pub(super) role_steps: Vec<RoleStep>,
    pub(super) remaining_synthesis_attempts: u32,
    pub(super) remaining_repair_attempts: u32,
    pub(super) fallback_chain: FallbackChain,
    pub(super) rotation_decisions: Vec<crate::RouterRotationDecision>,
}

impl RouterExecutionPlan {
    /// Returns the selected recipe identifier.
    #[must_use]
    pub fn recipe_id(&self) -> &str {
        self.recipe_id.as_str()
    }

    /// Returns the route family.
    #[must_use]
    pub const fn family(&self) -> crate::RouteFamily {
        self.family
    }

    /// Returns selected safe model labels.
    #[must_use]
    pub fn selected_models(&self) -> &[String] {
        self.selected_models.as_slice()
    }

    /// Returns the final output contract.
    #[must_use]
    pub const fn output_contract(&self) -> crate::OutputContract {
        self.output_contract
    }

    /// Returns the validated execution bounds.
    #[must_use]
    pub const fn bounds(&self) -> &crate::Bounds {
        &self.bounds
    }

    /// Returns the projected privacy band used for step eligibility.
    #[must_use]
    pub const fn privacy(&self) -> crate::PrivacyBand {
        self.privacy
    }

    /// Returns the projected fallback eligibility band.
    #[must_use]
    pub const fn fallback_eligibility(&self) -> crate::FallbackEligibility {
        self.fallback_eligibility
    }

    /// Returns ordered role steps.
    #[must_use]
    pub fn role_steps(&self) -> &[RoleStep] {
        self.role_steps.as_slice()
    }

    /// Returns remaining synthesis attempts.
    #[must_use]
    pub const fn remaining_synthesis_attempts(&self) -> u32 {
        self.remaining_synthesis_attempts
    }

    /// Returns remaining repair attempts.
    #[must_use]
    pub const fn remaining_repair_attempts(&self) -> u32 {
        self.remaining_repair_attempts
    }

    /// Returns the ordered fallback chain.
    #[must_use]
    pub fn fallback_chain(&self) -> &FallbackChain {
        &self.fallback_chain
    }

    /// Returns safe rotation decisions used during selection or fallback materialization.
    #[must_use]
    pub fn rotation_decisions(&self) -> &[crate::RouterRotationDecision] {
        self.rotation_decisions.as_slice()
    }
}

/// One ordered role step in a materialized router execution plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleStep {
    role: crate::Role,
    candidate_labels: Vec<String>,
    parallelism_cap: u32,
    quorum: u32,
}

impl RoleStep {
    /// Returns the role for this step.
    #[must_use]
    pub const fn role(&self) -> crate::Role {
        self.role
    }

    /// Returns the ordered candidate labels for this step.
    #[must_use]
    pub fn candidate_labels(&self) -> &[String] {
        self.candidate_labels.as_slice()
    }

    /// Returns the parallelism cap for this step.
    #[must_use]
    pub const fn parallelism_cap(&self) -> u32 {
        self.parallelism_cap
    }

    /// Returns the quorum threshold for this step.
    #[must_use]
    pub const fn quorum(&self) -> u32 {
        self.quorum
    }
}

/// Ordered fallback chain materialized from the catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FallbackChain {
    pub(super) steps: Vec<FallbackStep>,
}

impl FallbackChain {
    /// Returns the ordered fallback steps.
    #[must_use]
    pub fn steps(&self) -> &[FallbackStep] {
        self.steps.as_slice()
    }
}

/// One fallback step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FallbackStep {
    pub(super) role: crate::Role,
    pub(super) model_label: String,
    pub(super) backend: String,
    pub(super) local: bool,
}

impl FallbackStep {
    /// Returns the fallback role.
    #[must_use]
    pub const fn role(&self) -> crate::Role {
        self.role
    }

    /// Returns the fallback model label.
    #[must_use]
    pub fn model_label(&self) -> &str {
        self.model_label.as_str()
    }

    /// Returns the fallback backend label.
    #[must_use]
    pub fn backend(&self) -> &str {
        self.backend.as_str()
    }

    /// Returns whether the fallback model is local-only.
    #[must_use]
    pub const fn is_local(&self) -> bool {
        self.local
    }
}

/// Typed router execution-plan denial.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionPlanError {
    /// The quorum threshold was zero.
    ZeroQuorum,
    /// The quorum threshold exceeded the available candidate labels.
    ImpossibleQuorum,
    /// Collaborative repair budget was not available.
    UnboundedRepair,
    /// A recipe role could not be mapped to a selected model step.
    MissingRoleMapping,
    /// The fallback chain revisited a model already used in the route.
    CyclicFallback,
    /// No fallback model satisfied backend or privacy constraints.
    FallbackModelNotEligible,
}
