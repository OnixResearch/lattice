//! `lattice.worker-profile-facts.v2`: bounded, material-free profile facts.
//!
//! Profile owners stay where they are: `ConfiguredBinding.revision` for
//! configured routes and the implementation-attempt `worker_profile_identity`
//! for the OMP attempt worker. Commands, argv, prompts, endpoints, credential
//! values and physical paths are never inputs here. The definition identity,
//! declared capability, #25 revision slots and wait reasons are projected
//! here from owner-supplied facts only.

// r[impl lattice.execution_binding.profile_facts]
// r[impl lattice.execution_binding.profile_facts.scenario.projection]
// r[impl lattice.execution_binding.deferred.scenario.serving]
// r[impl lattice.worker_inventory.schema]
// r[impl lattice.worker_inventory.definition.scenario.stable]
// r[impl lattice.worker_inventory.definition.scenario.basis]
// r[impl lattice.worker_inventory.capability.scenario.declared]
// r[impl lattice.worker_inventory.capability.scenario.undeclared]
// r[impl lattice.worker_inventory.revisions]
// r[impl lattice.worker_inventory.revisions.scenario.unpublished]

pub const PROFILE_FACTS_SCHEMA: &str = "lattice.worker-profile-facts.v2";
/// Owner-default definition name of the OMP implementation-attempt worker.
pub const ATTEMPT_DEFINITION: &str = "omp-attempt-worker";

type Code = super::wait::Code;
type Field = super::wait::FactField;
type Reason = super::wait::Reason;

/// Which existing profile owner supplied the revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileOwner {
    /// `lattice.configured-agent-runtime.v1` configured Pi, Clankers or Animus routes.
    ConfiguredRuntime,
    /// `lattice.implementation-attempt-run.v3` OMP attempt worker.
    ImplementationAttempt,
}

impl ProfileOwner {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ConfiguredRuntime => "configured_runtime",
            Self::ImplementationAttempt => "implementation_attempt",
        }
    }
}

/// Owner-stated facts, borrowed from the owner's own projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileInput<'a> {
    pub owner: ProfileOwner,
    pub revision: &'a str,
    /// The owner's definition name when no capability is declared.
    pub default_definition: &'a str,
    /// The worker configuration's optional capability declaration.
    pub capability: Option<&'a super::super::capability::Declaration>,
    /// Bound qualification evidence; nothing supplies it yet (Animus #18).
    pub qualification_ref: Option<&'a str>,
    pub adapter: super::Offered<&'a str>,
    pub protocol_revision: super::Offered<&'a str>,
    pub executable_identity: super::Offered<&'a str>,
    pub inputs: super::Offered<&'a str>,
    pub outputs: super::Offered<&'a str>,
    pub sessions: super::Offered<&'a str>,
    pub checkpoint: super::Offered<&'a str>,
    pub workspace: super::Offered<&'a str>,
    pub network: super::Offered<&'a str>,
    pub isolation: super::Offered<&'a str>,
    pub deadline_secs: super::Offered<u64>,
    pub max_output_bytes: super::Offered<u64>,
    pub model_routes: super::Offered<&'a super::super::request::RouteDeclaration>,
    pub compute: super::Offered<&'a super::super::request::ComputeRequirement>,
    /// Readiness from the injected serving-owner port, or why none was observed.
    pub serving_readiness: &'a super::ServingReadiness,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerProfileFacts {
    pub schema: String,
    pub owner: ProfileOwner,
    pub definition: super::lineage::Definition,
    pub revision: String,
    pub qualification_ref: Option<String>,
    pub adapter: super::Fact<String>,
    pub protocol_revision: super::Fact<String>,
    pub executable_identity: super::Fact<String>,
    pub inputs: super::Fact<String>,
    pub outputs: super::Fact<String>,
    pub sessions: super::Fact<String>,
    pub checkpoint: super::Fact<String>,
    pub workspace: super::Fact<String>,
    pub network: super::Fact<String>,
    pub isolation: super::Fact<String>,
    pub deadline_secs: super::Fact<u64>,
    pub max_output_bytes: super::Fact<u64>,
    /// Lattice enforces no CPU, memory or storage bound; always unknown.
    pub cpu_millis: super::Fact<u64>,
    pub memory_bytes: super::Fact<u64>,
    pub storage_bytes: super::Fact<u64>,
    /// Sorted declared names; `unknown` when undeclared, never empty.
    pub task_families: super::Fact<Vec<String>>,
    pub tools: super::Fact<Vec<String>>,
    pub model_routes: super::Fact<super::super::request::RouteDeclaration>,
    pub compute: super::Fact<super::super::request::ComputeRequirement>,
    pub serving_readiness: super::ServingReadiness,
    pub revisions: super::lineage::RevisionSlots,
    pub wait_reasons: Vec<Reason>,
}

/// A fact the projection refuses to publish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "defect", rename_all = "snake_case")]
pub enum FactsDefect {
    Revision,
    Token {
        field: &'static str,
    },
    Routes,
    Definition,
    Capability {
        capability: super::super::capability::Defect,
    },
    WaitReasons,
}

/// Project one profile. `qualified` appears only with canonical evidence.
pub fn project_profile(input: &ProfileInput<'_>) -> Result<WorkerProfileFacts, FactsDefect> {
    if !super::super::is_token(input.revision) {
        return Err(FactsDefect::Revision);
    }
    if !input.serving_readiness.is_bounded() {
        return Err(FactsDefect::Routes);
    }
    let capability = input
        .capability
        .map(super::super::capability::Declaration::canonical)
        .transpose()
        .map_err(|capability| FactsDefect::Capability { capability })?;
    let qualification_ref = input.qualification_ref.filter(|value| super::super::is_reference(value));
    let is_qualified = qualification_ref.is_some();
    let text = |offered: super::Offered<&str>, field| text_fact(offered, is_qualified, field);
    let number = |offered: super::Offered<u64>| offered.project(is_qualified, std::convert::identity);
    let names = |set: Option<&super::super::capability::NameSet>| names_fact(set, is_qualified);
    let facts = WorkerProfileFacts {
        schema: PROFILE_FACTS_SCHEMA.to_owned(),
        owner: input.owner,
        definition: super::lineage::definition(
            input.owner,
            input.default_definition,
            capability.as_ref().map(|declared| declared.definition.as_str()),
        )
        .map_err(|_| FactsDefect::Definition)?,
        revision: input.revision.to_owned(),
        qualification_ref: qualification_ref.map(str::to_owned),
        adapter: text(input.adapter, "adapter")?,
        protocol_revision: text(input.protocol_revision, "protocol_revision")?,
        executable_identity: text(input.executable_identity, "executable_identity")?,
        inputs: text(input.inputs, "inputs")?,
        outputs: text(input.outputs, "outputs")?,
        sessions: text(input.sessions, "sessions")?,
        checkpoint: text(input.checkpoint, "checkpoint")?,
        workspace: text(input.workspace, "workspace")?,
        network: text(input.network, "network")?,
        isolation: text(input.isolation, "isolation")?,
        deadline_secs: number(input.deadline_secs),
        max_output_bytes: number(input.max_output_bytes),
        cpu_millis: super::Offered::Unknown.project(false, std::convert::identity),
        memory_bytes: super::Offered::Unknown.project(false, std::convert::identity),
        storage_bytes: super::Offered::Unknown.project(false, std::convert::identity),
        task_families: names(capability.as_ref().and_then(|declared| declared.task_families.as_ref())),
        tools: names(capability.as_ref().and_then(|declared| declared.tools.as_ref())),
        model_routes: routes_fact(input.model_routes, is_qualified)?,
        compute: input.compute.project(is_qualified, Clone::clone),
        serving_readiness: input.serving_readiness.clone(),
        revisions: super::lineage::RevisionSlots::unpublished(),
        wait_reasons: Vec::new(),
    };
    debug_assert_eq!(facts.cpu_millis.support, super::SupportLevel::Unknown);
    debug_assert_eq!(facts.qualification_ref.is_some(), is_qualified);
    Ok(WorkerProfileFacts {
        wait_reasons: profile_reasons(&facts)?,
        ..facts
    })
}

/// Declared (or qualified) sorted names, `unavailable`, or `unknown` when absent.
fn names_fact(set: Option<&super::super::capability::NameSet>, is_qualified: bool) -> super::Fact<Vec<String>> {
    match set {
        Some(super::super::capability::NameSet::Declared { names }) => {
            super::Offered::Declared(names).project(is_qualified, Clone::clone)
        }
        Some(super::super::capability::NameSet::Unavailable {}) => {
            super::Offered::<&Vec<String>>::Unavailable.project(false, Clone::clone)
        }
        None => super::Offered::<&Vec<String>>::Unknown.project(false, Clone::clone),
    }
}

/// Why this profile cannot yet be relied on, from its own facts only.
fn profile_reasons(facts: &WorkerProfileFacts) -> Result<Vec<Reason>, FactsDefect> {
    let mut reasons = Vec::with_capacity(super::wait::MAX_WAIT_REASONS);
    for (fact, field) in [
        (&facts.task_families, Field::TaskFamilies),
        (&facts.tools, Field::Tools),
    ] {
        if fact.support == super::SupportLevel::Unknown {
            reasons.push(Reason::fact(Code::CapabilityUndeclared, field));
        }
    }
    if facts.qualification_ref.is_none() {
        reasons.push(Reason::fact(Code::QualificationUnbound, Field::QualificationRef));
    }
    reasons.push(Reason::fact(Code::RevisionSelectionUnpublished, Field::Revisions));
    debug_assert!(reasons.len() <= 4, "two capability sets, qualification and revisions");
    debug_assert!(facts.serving_readiness.is_bounded());
    reasons.extend(super::wait::serving(&facts.serving_readiness));
    super::wait::finish(reasons).map_err(|_| FactsDefect::WaitReasons)
}

fn text_fact(
    offered: super::Offered<&str>,
    is_qualified: bool,
    field: &'static str,
) -> Result<super::Fact<String>, FactsDefect> {
    if let super::Offered::Declared(value) = offered
        && !super::super::is_token(value)
    {
        return Err(FactsDefect::Token { field });
    }
    Ok(offered.project(is_qualified, str::to_owned))
}

fn routes_fact(
    offered: super::Offered<&super::super::request::RouteDeclaration>,
    is_qualified: bool,
) -> Result<super::Fact<super::super::request::RouteDeclaration>, FactsDefect> {
    if let super::Offered::Declared(super::super::request::RouteDeclaration::Routes { routes }) = offered
        && routes.len() > super::super::MAX_ROUTES
    {
        return Err(FactsDefect::Routes);
    }
    Ok(offered.project(is_qualified, Clone::clone))
}
