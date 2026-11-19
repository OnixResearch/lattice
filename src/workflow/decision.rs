//! Pure typed policy operations: one bounded selection among a Condition node's declared branches.
//!
//! This functional core admits versioned operation profiles, binds them to declared Condition
//! nodes, projects bounded integer features, validates frozen-policy receipts independently,
//! resolves typed outcomes through the declared fallback, and builds and replays durable decision
//! records. It performs no inference, artifact loading, file, clock, process or network I/O; the
//! Animus adapter shell (`crate::animus_agent::frozen`) is the only inference site.

// r[impl lattice.typed_policy.profile]
pub mod admission;
// r[impl lattice.typed_policy.route]
pub mod binding;
pub mod checks;
// r[impl lattice.typed_policy.record]
pub mod ledger;
// r[impl lattice.typed_policy.validation]
pub mod validation;

const IDENTITY_PREFIX: &str = "b3:";
const DOMAIN_SEPARATOR: &[u8] = &[0];

/// The `b3:` identity of `fields` under `domain`; the fields are framed as one JSON array.
#[must_use]
pub fn identity(domain: &str, fields: &[&str]) -> String {
    let framed = serde_json::Value::Array(fields.iter().map(|field| serde_json::Value::from(*field)).collect());
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(DOMAIN_SEPARATOR);
    hasher.update(framed.to_string().as_bytes());
    format!("{IDENTITY_PREFIX}{}", hasher.finalize().to_hex())
}

/// The response of one adapter invocation, before Lattice validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// The raw receipt bytes the adapter returned.
    Receipt(Vec<u8>),
    /// The backend could not be reached or loaded.
    Unavailable,
}

/// The inference boundary. Implementations live in shells; the core never calls one.
pub trait Adapter: Send + Sync {
    /// Runs one frozen-policy decision for `invocation`.
    fn invoke(&self, invocation: &binding::Invocation) -> Response;
}

/// Admitted profiles and the adapter an executor uses for bound Condition nodes.
#[derive(Clone)]
pub struct Runtime {
    /// Every admitted profile; disabled profiles never bind.
    pub profiles: Vec<admission::Admitted>,
    /// The inference adapter.
    pub adapter: std::sync::Arc<dyn Adapter>,
}

impl std::fmt::Debug for Runtime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Runtime").field("profiles", &self.profiles).finish_non_exhaustive()
    }
}

impl Runtime {
    /// The enabled profile bound to `node` of `workflow`, if any.
    #[must_use]
    pub fn profile_for(
        &self,
        workflow: &crate::engine::workflow::Workflow,
        node: &crate::engine::workflow::Node,
    ) -> Option<&admission::Admitted> {
        self.profiles.iter().find(|admitted| admitted.selects(workflow, node))
    }
}
