//! `lattice.agent-bundle.v1`: the inert, material-free facts of one resolved
//! revision.
//!
//! The shell resolves a bundle from an exported implementation-attempt
//! request. A bundle carries identities and declared names only: no paths,
//! prompts, endpoints, credential values or commands. Credential references
//! are bound only through each route identity. Hosted model routes carry the
//! provider-exposed model selector; weight and version identity are always
//! `unknown`, and the type admits no other value.

// r[impl lattice.agent_revisions.bundle]
// r[impl lattice.agent_revisions.bundle.scenario.hosted]

pub const BUNDLE_SCHEMA: &str = "lattice.agent-bundle.v1";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub schema: String,
    /// `blake3:` definition identity.
    pub definition: String,
    pub definition_name: String,
    /// Bare 64-hex profile revision, verbatim from its owner.
    pub revision: String,
    pub adapter: String,
    pub protocol_revision: String,
    pub executable_identity: String,
    pub compute: crate::execution_binding::request::ComputeRequirement,
    /// Declared, sorted; `None` is undeclared, never empty.
    pub task_families: Option<Vec<String>>,
    pub tools: Option<Vec<String>>,
    pub routes: Vec<HostedRoute>,
    /// Digest of declared section 1 components, when declared.
    pub components: Option<String>,
}

/// A brokered hosted model route.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostedRoute {
    pub name: String,
    /// `lattice-model-route-v1` identity over schema, endpoint, model and
    /// credential reference.
    pub route_identity: String,
    /// The provider-exposed selector; an endpoint name is not a weight identity.
    pub provider_model: String,
    pub weight_identity: Unexposed,
    pub model_version: Unexposed,
}

/// A fact the hosted provider does not expose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unexposed {
    Unknown,
}

/// Check the schema, identities and bounded names.
pub fn validate(bundle: &Bundle) -> Result<(), super::Refusal> {
    let mut names = bundle.task_families.iter().chain(bundle.tools.iter()).flatten();
    let mut routes = bundle.routes.iter();
    let texts = [
        &bundle.definition_name,
        &bundle.adapter,
        &bundle.protocol_revision,
        &bundle.executable_identity,
    ];
    let checks = [
        bundle.schema == BUNDLE_SCHEMA,
        super::is_reference(&bundle.definition),
        super::is_bare_digest(&bundle.revision),
        texts.iter().all(|value| super::is_token(value)),
        names.all(|name| crate::execution_binding::capability::is_name(name)),
        routes.all(is_route),
        bundle.components.as_deref().is_none_or(super::is_reference),
        super::is_within(bundle.routes.len(), super::MAX_REFERENCES),
    ];
    if checks.iter().all(|check| *check) {
        debug_assert!(super::is_reference(&bundle.definition));
        debug_assert!(super::is_bare_digest(&bundle.revision));
        Ok(())
    } else {
        Err(super::Refusal::Malformed)
    }
}

fn is_route(route: &HostedRoute) -> bool {
    let tokens = [&route.name, &route.provider_model];
    tokens.iter().all(|value| super::is_token(value)) && super::is_reference(&route.route_identity)
}

/// Validate and return the bundle's `lattice-agent-bundle-v1` identity.
pub fn identity(bundle: &Bundle) -> Result<String, super::Refusal> {
    validate(bundle)?;
    let canonical = serde_json::to_vec(bundle).map_err(|_| super::Refusal::Malformed)?;
    super::identity::bundle(&canonical).map_err(|_| super::Refusal::Malformed)
}
