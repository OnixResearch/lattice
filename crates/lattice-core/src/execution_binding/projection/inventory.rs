//! `lattice.worker-profile-facts.v1`: bounded, material-free profile facts.
//!
//! Profile owners stay where they are: `ConfiguredBinding.revision` for
//! configured routes and the implementation-attempt `worker_profile_identity`
//! for the OMP attempt worker. Commands, argv, prompts, endpoints, credential
//! values and physical paths are never inputs here.

// r[impl lattice.execution_binding.profile_facts]
// r[impl lattice.execution_binding.profile_facts.scenario.projection]
// r[impl lattice.execution_binding.deferred.scenario.serving]

pub const PROFILE_FACTS_SCHEMA: &str = "lattice.worker-profile-facts.v1";

/// Which existing profile owner supplied the revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileOwner {
    /// `lattice.configured-agent-runtime.v1` configured Pi, Clankers or Animus routes.
    ConfiguredRuntime,
    /// `lattice.implementation-attempt-run.v3` OMP attempt worker.
    ImplementationAttempt,
}

/// Owner-stated facts, borrowed from the owner's own projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileInput<'a> {
    pub owner: ProfileOwner,
    pub revision: &'a str,
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
    pub model_routes: super::Fact<super::super::request::RouteDeclaration>,
    pub compute: super::Fact<super::super::request::ComputeRequirement>,
    pub serving_readiness: super::ServingReadiness,
}

/// A fact the projection refuses to publish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "defect", rename_all = "snake_case")]
pub enum FactsDefect {
    Revision,
    Token { field: &'static str },
    Routes,
}

/// Project one profile. `qualified` appears only with canonical evidence.
pub fn project_profile(input: &ProfileInput<'_>) -> Result<WorkerProfileFacts, FactsDefect> {
    if !super::super::is_token(input.revision) {
        return Err(FactsDefect::Revision);
    }
    if !input.serving_readiness.is_bounded() {
        return Err(FactsDefect::Routes);
    }
    let qualification_ref = input.qualification_ref.filter(|value| super::super::is_reference(value));
    let is_qualified = qualification_ref.is_some();
    let text = |offered: super::Offered<&str>, field| text_fact(offered, is_qualified, field);
    let number = |offered: super::Offered<u64>| offered.project(is_qualified, std::convert::identity);
    let facts = WorkerProfileFacts {
        schema: PROFILE_FACTS_SCHEMA.to_owned(),
        owner: input.owner,
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
        model_routes: routes_fact(input.model_routes, is_qualified)?,
        compute: input.compute.project(is_qualified, Clone::clone),
        serving_readiness: input.serving_readiness.clone(),
    };
    debug_assert_eq!(facts.cpu_millis.support, super::SupportLevel::Unknown);
    debug_assert_eq!(facts.qualification_ref.is_some(), is_qualified);
    Ok(facts)
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
