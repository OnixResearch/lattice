//! `lattice.worker-profile-facts.v3`: bounded, material-free profile facts.
//!
//! Profile owners stay where they are: `ConfiguredBinding.revision` for
//! configured routes and the implementation-attempt `worker_profile_identity`
//! for the OMP attempt worker. Commands, argv, prompts, endpoints, credential
//! values and physical paths are never inputs here. The definition identity,
//! declared capability, revision slots, capability qualifications and wait
//! reasons are projected here from owner-supplied facts and the definition's
//! revision lifecycle only.

// r[impl lattice.execution_binding.profile_facts]
// r[impl lattice.execution_binding.profile_facts.scenario.projection]
// r[impl lattice.execution_binding.deferred.scenario.serving]
// r[impl lattice.worker_inventory.schema]
// r[impl lattice.worker_inventory.definition.scenario.stable]
// r[impl lattice.worker_inventory.definition.scenario.basis]
// r[impl lattice.worker_inventory.capability.scenario.declared]
// r[impl lattice.worker_inventory.capability.scenario.undeclared]
// r[impl lattice.worker_inventory.revisions]
// r[impl lattice.worker_inventory.qualification]
// r[impl lattice.worker_inventory.qualification.scenario.qualified]
// r[impl lattice.worker_inventory.qualification.scenario.unbound]

pub const PROFILE_FACTS_SCHEMA: &str = "lattice.worker-profile-facts.v3";
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
    /// The definition's revision lifecycle; `None` when no revision store was
    /// supplied or the definition is not lifecycle-managed.
    pub lifecycle: Option<&'a crate::agent_revision::state::Lifecycle>,
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
    /// Sorted declared names; `unknown` when undeclared, never empty;
    /// `qualified` when every declared family has a current qualification.
    pub task_families: super::Fact<Vec<String>>,
    pub tools: super::Fact<Vec<String>>,
    /// Current lifecycle qualifications of this revision, by task family.
    pub capability_qualifications: Vec<CapabilityQualification>,
    pub model_routes: super::Fact<super::super::request::RouteDeclaration>,
    pub compute: super::Fact<super::super::request::ComputeRequirement>,
    pub serving_readiness: super::ServingReadiness,
    pub revisions: super::lineage::RevisionSlots,
    pub wait_reasons: Vec<Reason>,
}

/// Qualified support for one task family: a current (unrevoked) lifecycle
/// qualification of the profile's own revision for that family.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityQualification {
    pub task_family: String,
    /// The Animus #18 attachment identity, verbatim.
    pub qualification_ref: String,
    pub evidence_class: crate::agent_revision::ports::EvidenceClass,
    /// `fixture` or `live`, as the producer labelled the attachment.
    pub provenance: crate::agent_revision::ports::Provenance,
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
    /// The lifecycle belongs to another definition.
    Lifecycle,
    Capability {
        capability: super::super::capability::Defect,
    },
    WaitReasons,
}

/// Project one profile. `qualified` appears only with a current lifecycle
/// qualification of this revision.
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
    let definition = super::lineage::definition(
        input.owner,
        input.default_definition,
        capability.as_ref().map(|declared| declared.definition.as_str()),
    )
    .map_err(|_| FactsDefect::Definition)?;
    if input.lifecycle.is_some_and(|lifecycle| lifecycle.definition != definition.identity) {
        return Err(FactsDefect::Lifecycle);
    }
    let qualifications = super::lineage::capability_qualifications(input.lifecycle, input.revision);
    let declared_families = capability.as_ref().and_then(|declared| declared.task_families.as_ref());
    let is_family_qualified = match declared_families {
        Some(super::super::capability::NameSet::Declared { names }) => {
            names.iter().all(|name| qualifications.iter().any(|qualified| qualified.task_family == *name))
        }
        Some(super::super::capability::NameSet::Unavailable {}) | None => false,
    };
    let text = |offered: super::Offered<&str>, field| text_fact(offered, field);
    let number = |offered: super::Offered<u64>| offered.project(false, std::convert::identity);
    let facts = WorkerProfileFacts {
        schema: PROFILE_FACTS_SCHEMA.to_owned(),
        owner: input.owner,
        definition,
        revision: input.revision.to_owned(),
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
        task_families: names_fact(declared_families, is_family_qualified),
        tools: names_fact(capability.as_ref().and_then(|declared| declared.tools.as_ref()), false),
        capability_qualifications: qualifications,
        model_routes: routes_fact(input.model_routes)?,
        compute: input.compute.project(false, Clone::clone),
        serving_readiness: input.serving_readiness.clone(),
        revisions: super::lineage::RevisionSlots::project(input.lifecycle),
        wait_reasons: Vec::new(),
    };
    debug_assert_eq!(facts.cpu_millis.support, super::SupportLevel::Unknown);
    debug_assert_eq!(facts.revisions.is_managed(), input.lifecycle.is_some());
    let is_admitting_new_sessions =
        input.lifecycle.is_none_or(|lifecycle| super::lineage::admits(lifecycle, input.revision));
    Ok(WorkerProfileFacts {
        wait_reasons: profile_reasons(&facts, is_admitting_new_sessions)?,
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
fn profile_reasons(facts: &WorkerProfileFacts, is_admitting_new_sessions: bool) -> Result<Vec<Reason>, FactsDefect> {
    let mut reasons = Vec::with_capacity(super::wait::MAX_WAIT_REASONS);
    for (fact, field) in [
        (&facts.task_families, Field::TaskFamilies),
        (&facts.tools, Field::Tools),
    ] {
        if fact.support == super::SupportLevel::Unknown {
            reasons.push(Reason::fact(Code::CapabilityUndeclared, field));
        }
    }
    if facts.task_families.support != super::SupportLevel::Qualified {
        reasons.push(Reason::fact(Code::QualificationUnbound, Field::CapabilityQualifications));
    }
    if !is_admitting_new_sessions {
        reasons.push(Reason::fact(Code::RevisionNotAdmitted, Field::Revisions));
    }
    debug_assert!(reasons.len() <= 4, "two capability sets, qualification and revisions");
    debug_assert!(facts.serving_readiness.is_bounded());
    reasons.extend(super::wait::serving(&facts.serving_readiness));
    super::wait::finish(reasons).map_err(|_| FactsDefect::WaitReasons)
}

fn text_fact(offered: super::Offered<&str>, field: &'static str) -> Result<super::Fact<String>, FactsDefect> {
    if let super::Offered::Declared(value) = offered
        && !super::super::is_token(value)
    {
        return Err(FactsDefect::Token { field });
    }
    Ok(offered.project(false, str::to_owned))
}

fn routes_fact(
    offered: super::Offered<&super::super::request::RouteDeclaration>,
) -> Result<super::Fact<super::super::request::RouteDeclaration>, FactsDefect> {
    if let super::Offered::Declared(super::super::request::RouteDeclaration::Routes { routes }) = offered
        && routes.len() > super::super::MAX_ROUTES
    {
        return Err(FactsDefect::Routes);
    }
    Ok(offered.project(false, Clone::clone))
}
