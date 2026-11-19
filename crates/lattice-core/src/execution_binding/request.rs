//! Operator-declared worker requirements and the per-attempt declaration.
//!
//! The shapes match `contracts/worker-execution-binding.ncl` and the
//! `worker_compute` and `execution_binding` fields of an implementation-attempt
//! v3 request. Validation is structural; admission compares them.

/// Schema of the per-attempt declaration.
pub const DECLARATION_SCHEMA: &str = "lattice.execution-binding.v1";

/// The compute a worker profile requires.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ComputeRequirement {
    /// Runs on the dispatching host; no provider allocation exists.
    HostLocal {},
    /// Needs exactly one provider execution allocation.
    ProviderExecution {
        direct_accelerator: Option<DirectAccelerator>,
    },
}

/// A directly allocated accelerator; the serving reference never satisfies it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectAccelerator {
    pub min_chips: u64,
    pub accepted_granularities: Vec<super::model::Granularity>,
    pub backend: Option<String>,
    pub runtime: Option<String>,
    pub architecture: Option<String>,
    pub minimum_enforcement: super::model::Enforcement,
}

/// The model routes a worker profile requires.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ModelRequirement {
    NoModel {},
    Routes { names: Vec<String> },
}

/// Compute and model requirements of one worker profile.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerRequirement {
    pub compute: ComputeRequirement,
    pub model: ModelRequirement,
}

/// Per-attempt declaration of the expected compute and model routes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Declaration {
    pub schema: String,
    pub compute: ComputeDeclaration,
    pub model_routes: RouteDeclaration,
    /// Optional serving-owner binding of named routes; absent keeps the
    /// binding's prior identity and leaves request usage unknown.
    #[serde(default = "super::serving::absent", skip_serializing_if = "Option::is_none")]
    pub serving: Option<super::serving::Declaration>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ComputeDeclaration {
    HostLocal {},
    /// The dispatcher-declared expectation. `owner_generation` is the
    /// producer owner generation, never Lattice's fence.
    ProviderAllocation {
        allocation_id: String,
        host_id: String,
        consumer_id: String,
        owner_generation: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RouteDeclaration {
    NoModel {},
    Routes { routes: Vec<NamedRoute> },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedRoute {
    pub name: String,
    /// `blake3:` route identity; never the endpoint text.
    pub route_identity: String,
}

/// Borrowed attempt identities the binding covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttemptContext<'a> {
    pub work_id: &'a str,
    pub attempt_id: &'a str,
    pub fence: u64,
    pub envelope_identity: &'a str,
    pub profile_revision: &'a str,
}

/// A structural defect in a requirement or declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "defect", rename_all = "snake_case")]
pub enum DeclarationDefect {
    Schema,
    Token { field: &'static str },
    Zero { field: &'static str },
    Routes,
    RouteIdentity,
    Granularities,
}

/// Check the declaration's schema, tokens, generation and routes.
pub fn validate_declaration(declaration: &Declaration) -> Result<(), DeclarationDefect> {
    if declaration.schema != DECLARATION_SCHEMA {
        return Err(DeclarationDefect::Schema);
    }
    if let ComputeDeclaration::ProviderAllocation {
        allocation_id,
        host_id,
        consumer_id,
        owner_generation,
    } = &declaration.compute
    {
        for (value, field) in [
            (allocation_id, "allocation_id"),
            (host_id, "host_id"),
            (consumer_id, "consumer_id"),
        ] {
            require(super::is_token(value), DeclarationDefect::Token { field })?;
        }
        require(*owner_generation > 0, DeclarationDefect::Zero {
            field: "owner_generation",
        })?;
    }
    debug_assert_eq!(declaration.schema, DECLARATION_SCHEMA);
    debug_assert!(super::is_token(&declaration.schema), "the declaration schema is a bounded token");
    match &declaration.model_routes {
        RouteDeclaration::NoModel {} => Ok(()),
        RouteDeclaration::Routes { routes } => validate_routes(routes),
    }
}

fn validate_routes(routes: &[NamedRoute]) -> Result<(), DeclarationDefect> {
    validate_names(routes.iter().map(|route| route.name.as_str()))?;
    let is_canonical = routes.iter().all(|route| super::is_reference(&route.route_identity));
    require(is_canonical, DeclarationDefect::RouteIdentity)
}

fn validate_names<'a>(names: impl ExactSizeIterator<Item = &'a str> + Clone) -> Result<(), DeclarationDefect> {
    let is_bounded = (1..=super::MAX_ROUTES).contains(&names.len());
    require(is_bounded, DeclarationDefect::Routes)?;
    require(names.clone().all(super::is_token), DeclarationDefect::Token { field: "route_name" })?;
    require(super::is_distinct(names), DeclarationDefect::Routes)
}

/// Check accelerator bounds and route names of one worker requirement.
pub fn validate_requirement(requirement: &WorkerRequirement) -> Result<(), DeclarationDefect> {
    if let ComputeRequirement::ProviderExecution {
        direct_accelerator: Some(accelerator),
    } = &requirement.compute
    {
        validate_accelerator(accelerator)?;
    }
    match &requirement.model {
        ModelRequirement::NoModel {} => Ok(()),
        ModelRequirement::Routes { names } => validate_names(names.iter().map(String::as_str)),
    }
}

fn validate_accelerator(accelerator: &DirectAccelerator) -> Result<(), DeclarationDefect> {
    require(accelerator.min_chips > 0, DeclarationDefect::Zero { field: "min_chips" })?;
    let granularities = &accelerator.accepted_granularities;
    let is_bounded = (1..=super::MAX_GRANULARITIES).contains(&granularities.len());
    let unique = granularities.iter().collect::<std::collections::BTreeSet<_>>();
    require(is_bounded && unique.len() == granularities.len(), DeclarationDefect::Granularities)?;
    for (value, field) in [
        (&accelerator.backend, "backend"),
        (&accelerator.runtime, "runtime"),
        (&accelerator.architecture, "architecture"),
    ] {
        require(value.as_deref().is_none_or(super::is_token), DeclarationDefect::Token { field })?;
    }
    Ok(())
}

fn require(condition: bool, defect: DeclarationDefect) -> Result<(), DeclarationDefect> {
    if condition { Ok(()) } else { Err(defect) }
}
