//! `lattice.serving-binding.v1`: the optional serving declaration of one
//! execution binding.
//!
//! The declaration names the Animus serving profile, privacy band and
//! attribution policy, and for each served named route the logical service,
//! endpoint identity, exact model and configuration and request requirements.
//! It is validated here, before any binding, intent or worker effect, with the
//! producer's opaque-token rule mirrored. Lattice is never the serving owner:
//! the declaration grants no capacity, endpoint, credential or tool.

// r[impl lattice.serving_binding.declaration]
// r[impl lattice.serving_binding.declaration.scenario.unsupported]

pub mod brokered;
pub mod envelope;
pub mod record;
pub mod usage;

pub const DECLARATION_SCHEMA: &str = "lattice.serving-binding.v1";
/// Producer bound on `max_snapshot_age_ticks` (`config/serving-capacity-profile.ncl`).
pub const MAX_SNAPSHOT_AGE_TICKS: u64 = 3600;
/// Producer opaque-token bound in bytes (`animus_adapters::serving::MAX_TOKEN_BYTES`).
const MAX_PRODUCER_TOKEN_BYTES: usize = 256;
/// Bytes the producer forbids in an opaque token besides whitespace and controls.
const FORBIDDEN_TOKEN_BYTES: &[u8] = b"/\\\"'`<>{}";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Declaration {
    pub schema: String,
    pub profile: Profile,
    pub privacy: Privacy,
    pub attribution: Attribution,
    pub routes: Vec<Route>,
    /// Refuse further requests once known served usage exceeds its declared
    /// token envelopes. Unset declarations keep their canonical bytes, so it
    /// is framed into the binding identity only when set.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub stop_on_overrun: bool,
}

/// Mirror of `animus.serving-capacity-profile.v1`; serving-owner admission
/// is always required by the producer.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub profile_id: String,
    pub identity_rule: IdentityRule,
    pub capacity_rule: CapacityRule,
    pub replica_rule: ReplicaRule,
    pub max_snapshot_age_ticks: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityRule {
    ClaimedAccepted,
    ObservedRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapacityRule {
    KnownRequired,
    OwnerAdmissionDecides,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplicaRule {
    SameModelAllowed,
    Pinned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Privacy {
    LocalOnly,
    RemoteAllowed,
}

/// How a shared deployment's resources relate to this attempt. Lattice never
/// computes an apportioned amount; a declared policy is only referenced.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Attribution {
    /// The shared deployment is referenced, never charged per worker.
    NotApportioned {},
    /// An explicit, versioned external policy; any amount it yields is an estimate.
    DeclaredEstimate { policy_schema: String, policy_ref: String },
}

/// One named model route served through a serving owner.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub name: String,
    pub service_id: String,
    /// Opaque endpoint identity; never a URL.
    pub endpoint_identity: String,
    pub model_id: String,
    pub configuration_ref: String,
    pub requirements: Requirements,
}

/// Request requirements screened against the serving envelope and capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirements {
    pub needs_tools: bool,
    pub needs_streaming: bool,
    pub needs_structured_output: bool,
    pub needs_developer_role: bool,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// A structural defect; the declaration is refused before effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "defect", rename_all = "snake_case")]
pub enum Defect {
    Schema,
    Token { field: &'static str },
    Reference { field: &'static str },
    SnapshotAge,
    Routes,
    UndeclaredRoute,
    OutputTokens,
    NoModel,
}

/// Validate the declaration against the binding's declared model routes.
pub fn validate(declaration: &Declaration, routes: &super::request::RouteDeclaration) -> Result<(), Defect> {
    require(declaration.schema == DECLARATION_SCHEMA, Defect::Schema)?;
    let super::request::RouteDeclaration::Routes { routes: declared } = routes else {
        return Err(Defect::NoModel);
    };
    validate_profile(&declaration.profile)?;
    validate_attribution(&declaration.attribution)?;
    let served = &declaration.routes;
    let is_bounded = (1..=super::MAX_ROUTES).contains(&served.len());
    require(is_bounded && super::is_distinct(served.iter().map(|route| route.name.as_str())), Defect::Routes)?;
    for route in served {
        let is_declared = declared.iter().any(|named| named.name == route.name);
        require(is_declared, Defect::UndeclaredRoute)?;
        validate_route(route)?;
    }
    debug_assert!(!served.is_empty(), "a validated declaration serves at least one route");
    debug_assert!(served.len() <= declared.len(), "every served route is a declared route");
    Ok(())
}

fn validate_profile(profile: &Profile) -> Result<(), Defect> {
    require(is_producer_token(&profile.profile_id), Defect::Token { field: "profile_id" })?;
    let is_age_bounded = (1..=MAX_SNAPSHOT_AGE_TICKS).contains(&profile.max_snapshot_age_ticks);
    require(is_age_bounded, Defect::SnapshotAge)
}

fn validate_attribution(attribution: &Attribution) -> Result<(), Defect> {
    let Attribution::DeclaredEstimate {
        policy_schema,
        policy_ref,
    } = attribution
    else {
        return Ok(());
    };
    require(is_producer_token(policy_schema), Defect::Token { field: "policy_schema" })?;
    require(super::is_reference(policy_ref), Defect::Reference { field: "policy_ref" })
}

fn validate_route(route: &Route) -> Result<(), Defect> {
    for (value, field) in [
        (&route.name, "route_name"),
        (&route.service_id, "service_id"),
        (&route.endpoint_identity, "endpoint_identity"),
        (&route.model_id, "model_id"),
    ] {
        require(is_producer_token(value), Defect::Token { field })?;
    }
    require(super::is_reference(&route.configuration_ref), Defect::Reference {
        field: "configuration_ref",
    })?;
    let requirements = route.requirements;
    let has_output = requirements.output_tokens > 0;
    let is_summable = requirements.input_tokens.checked_add(requirements.output_tokens).is_some();
    require(has_output && is_summable, Defect::OutputTokens)
}

/// The producer's opaque-token rule: 1 to 256 graphic ASCII bytes without
/// path, quote, backtick, angle-bracket or brace bytes.
#[must_use]
pub fn is_producer_token(value: &str) -> bool {
    let is_bounded = (1..=MAX_PRODUCER_TOKEN_BYTES).contains(&value.len());
    is_bounded && value.bytes().all(|byte| byte.is_ascii_graphic() && !FORBIDDEN_TOKEN_BYTES.contains(&byte))
}

/// The served route of `name`, if the declaration serves it.
#[must_use]
pub fn route<'a>(declaration: &'a Declaration, name: &str) -> Option<&'a Route> {
    declaration.routes.iter().find(|route| route.name == name)
}

/// Canonical bytes framed into the execution binding identity.
pub fn canonical(declaration: &Declaration) -> Result<String, serde_json::Error> {
    serde_json::to_string(declaration)
}

pub(crate) fn require(condition: bool, defect: Defect) -> Result<(), Defect> {
    if condition { Ok(()) } else { Err(defect) }
}

/// Structural validity of stored coverage against the provenance's routes.
/// `unknown` stays valid for every route shape: historical rows used it for
/// no-model workers too.
#[must_use]
pub fn is_valid_coverage(coverage: &usage::ModelRequestCoverage, routes: &super::request::RouteDeclaration) -> bool {
    match coverage {
        usage::ModelRequestCoverage::Unknown => true,
        usage::ModelRequestCoverage::NoModel => matches!(routes, super::request::RouteDeclaration::NoModel {}),
        usage::ModelRequestCoverage::Served(summary) => {
            let super::request::RouteDeclaration::Routes { routes } = routes else {
                return false;
            };
            let is_bounded = (1..=super::MAX_ROUTES).contains(&summary.services.len());
            let is_declared = summary.services.iter().all(|service| {
                service.requests.len() <= usage::MAX_REQUESTS_PER_ROUTE
                    && routes.iter().any(|route| route.name == service.route_name)
            });
            debug_assert!(!routes.is_empty(), "declared routes are nonempty");
            summary.schema == usage::USAGE_SCHEMA && is_bounded && is_declared
        }
        usage::ModelRequestCoverage::BrokeredUnattributed(summary) => {
            let super::request::RouteDeclaration::Routes { routes } = routes else {
                return false;
            };
            let is_declared =
                summary.requests.iter().all(|entry| routes.iter().any(|route| route.name == entry.route_name));
            summary.schema == brokered::USAGE_SCHEMA && summary.requests.len() <= brokered::MAX_REQUESTS && is_declared
        }
    }
}

/// An absent serving declaration; historical bindings decode to it.
#[must_use]
pub const fn absent() -> Option<Declaration> {
    None
}

/// The `lattice-model-route-v1` identity of a declared model route.
pub(crate) fn declared_route_identity(admitted: &super::admission::bound::Admitted, name: &str) -> Option<String> {
    let super::request::RouteDeclaration::Routes { routes } = &admitted.model_routes else {
        return None;
    };
    routes.iter().find(|route| route.name == name).map(|route| route.route_identity.clone())
}

/// A dispatch intent without an observation: the outcome and usage are unknown.
pub(crate) fn unconfirmed(intent: &record::DispatchIntent) -> usage::Entry {
    debug_assert!(!intent.grant_ref.is_empty(), "an intent records its grant");
    debug_assert!(intent.ordinal > 0, "ordinals start at one");
    usage::Entry {
        request_id: intent.request_id.clone(),
        ordinal: intent.ordinal,
        class: usage::EntryClass::DispatchUnconfirmed,
        outcome: "dispatch-unconfirmed".to_owned(),
        deployment_id: Some(intent.deployment_id.clone()),
        deployment_generation: Some(intent.deployment_generation),
        replica: None,
        resource_consumer_ref: None,
        usage: record::Usage::UNKNOWN,
        evidence: None,
        receipt_identity: None,
    }
}
