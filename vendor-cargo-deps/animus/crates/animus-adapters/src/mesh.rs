//! Pure Mesh LLM catalog, exact-model, authority, and receipt contracts.

// External catalog DTO fields stay compact, while independent admission booleans preserve denial provenance.
#![allow(missing_docs, clippy::struct_excessive_bools)]

use alloc::string::ToString;

const CATALOG_DOMAIN: &str = "animus.mesh.catalog.v1";
const RECEIPT_DOMAIN: &str = "animus.mesh.receipt.v1";
/// Maximum admitted catalog rows.
pub const MAX_CATALOG_MODELS: usize = 128;
/// Maximum safe model identifier length.
pub const MAX_MODEL_ID_BYTES: usize = 128;
const RECEIPT_NON_CLAIM_COUNT: usize = 6;

/// Explicit reviewed OpenAI-compatible API style.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApiStyle {
    Responses,
    ChatCompletions,
}

/// Conservative capability observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    Supported,
    Unsupported,
    Unknown,
}

/// Host-loaded raw model row; loading remains a shell effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogRow<'a> {
    pub id: &'a str,
    pub text: Capability,
    pub tools: Capability,
    pub streaming: Capability,
    pub structured_output: Capability,
    pub developer_role: Capability,
    pub context_tokens: u32,
    pub output_tokens: u32,
}

/// Host-supplied catalog snapshot metadata.
pub struct CatalogSnapshot<'a> {
    pub endpoint_identity: &'a str,
    pub config_identity: &'a str,
    pub freshness_identity: &'a str,
    pub private_gateway: bool,
    pub rows: &'a [CatalogRow<'a>],
}

/// Validated deterministic model row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    pub id: alloc::string::String,
    pub text: Capability,
    pub tools: Capability,
    pub streaming: Capability,
    pub structured_output: Capability,
    pub developer_role: Capability,
    pub context_tokens: u32,
    pub output_tokens: u32,
}

/// Validated deterministic catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Catalog {
    pub endpoint_identity: alloc::string::String,
    pub config_identity: alloc::string::String,
    pub freshness_identity: alloc::string::String,
    pub identity: alloc::string::String,
    pub models: alloc::vec::Vec<Model>,
}

/// Catalog normalization failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogError {
    Empty,
    TooManyRows,
    UnsafeIdentity,
    UnsafeModelId,
    DuplicateModel,
    PublicGateway,
    InvalidBounds,
}

/// Purely normalizes an already-loaded private-gateway catalog.
///
/// # Errors
///
/// Returns [`CatalogError`] for unsafe, ambiguous, public, duplicate, or
/// out-of-bounds catalog data.
pub fn normalize_catalog(snapshot: &CatalogSnapshot<'_>) -> Result<Catalog, CatalogError> {
    if snapshot.rows.is_empty() {
        return Err(CatalogError::Empty);
    }
    if snapshot.rows.len() > MAX_CATALOG_MODELS {
        return Err(CatalogError::TooManyRows);
    }
    if !safe_identity(snapshot.endpoint_identity)
        || !safe_identity(snapshot.config_identity)
        || !safe_identity(snapshot.freshness_identity)
    {
        return Err(CatalogError::UnsafeIdentity);
    }
    if !snapshot.private_gateway {
        return Err(CatalogError::PublicGateway);
    }
    debug_assert!(!snapshot.rows.is_empty());
    debug_assert!(snapshot.rows.len() <= MAX_CATALOG_MODELS);
    debug_assert!(snapshot.private_gateway);

    let mut ids = alloc::collections::BTreeSet::new();
    let mut models = alloc::vec::Vec::with_capacity(snapshot.rows.len());
    for row in snapshot.rows {
        if !safe_model_id(row.id) {
            return Err(CatalogError::UnsafeModelId);
        }
        if !ids.insert(row.id) {
            return Err(CatalogError::DuplicateModel);
        }
        if row.context_tokens == 0 || row.output_tokens == 0 || row.output_tokens > row.context_tokens {
            return Err(CatalogError::InvalidBounds);
        }
        models.push(Model {
            id: row.id.to_string(),
            text: row.text,
            tools: row.tools,
            streaming: row.streaming,
            structured_output: row.structured_output,
            developer_role: row.developer_role,
            context_tokens: row.context_tokens,
            output_tokens: row.output_tokens,
        });
    }
    models.sort_by(|left, right| left.id.cmp(&right.id));
    debug_assert_eq!(models.len(), snapshot.rows.len());
    debug_assert!(models.windows(2).all(|pair| pair[0].id < pair[1].id));

    let identity = catalog_identity(snapshot, &models);
    debug_assert!(safe_identity(&identity));
    Ok(Catalog {
        endpoint_identity: snapshot.endpoint_identity.to_string(),
        config_identity: snapshot.config_identity.to_string(),
        freshness_identity: snapshot.freshness_identity.to_string(),
        identity,
        models,
    })
}

fn safe_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_MODEL_ID_BYTES
        && !value.contains('\0')
        && !value.contains('/')
        && !value.contains('\\')
}
fn safe_model_id(value: &str) -> bool {
    safe_identity(value)
        && value != "auto"
        && value != "mesh"
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn catalog_identity(snapshot: &CatalogSnapshot<'_>, models: &[Model]) -> alloc::string::String {
    let mut hasher = domain_hasher(CATALOG_DOMAIN);
    for value in [
        snapshot.endpoint_identity,
        snapshot.config_identity,
        snapshot.freshness_identity,
    ] {
        hash_part(&mut hasher, value);
    }
    for model in models {
        hash_part(&mut hasher, &model.id);
        hash_part(&mut hasher, &model.context_tokens.to_string());
        hash_part(&mut hasher, &model.output_tokens.to_string());
    }
    hasher.finalize().to_hex().to_string()
}

/// Required canonical request capabilities and bounds.
pub struct Requirements {
    pub tools: bool,
    pub streaming: bool,
    pub structured_output: bool,
    pub developer_role: bool,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Exact-model admission result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExactModelDecision {
    Admitted,
    MissingModel,
    VirtualModelDenied,
    CapabilityDenied,
    BudgetDenied,
}

/// Admits only one exact model and conservative explicit capabilities.
#[must_use]
pub fn admit_exact_model(catalog: &Catalog, model_id: &str, requirements: &Requirements) -> ExactModelDecision {
    if model_id == "auto" || model_id == "mesh" || model_id.is_empty() {
        return ExactModelDecision::VirtualModelDenied;
    }
    let Some(model) = catalog.models.iter().find(|model| model.id == model_id) else {
        return ExactModelDecision::MissingModel;
    };
    debug_assert_eq!(model.id, model_id);
    debug_assert!(!model_id.is_empty());

    if model.text != Capability::Supported {
        return ExactModelDecision::CapabilityDenied;
    }
    if requirements.tools && model.tools != Capability::Supported {
        return ExactModelDecision::CapabilityDenied;
    }
    if requirements.streaming && model.streaming != Capability::Supported {
        return ExactModelDecision::CapabilityDenied;
    }
    if requirements.structured_output && model.structured_output != Capability::Supported {
        return ExactModelDecision::CapabilityDenied;
    }
    if requirements.developer_role && model.developer_role != Capability::Supported {
        return ExactModelDecision::CapabilityDenied;
    }
    let requested_tokens = requirements.input_tokens.saturating_add(requirements.output_tokens);
    if requirements.output_tokens > model.output_tokens || requested_tokens > model.context_tokens {
        return ExactModelDecision::BudgetDenied;
    }
    debug_assert!(requirements.output_tokens <= model.output_tokens);
    debug_assert!(requested_tokens <= model.context_tokens);
    ExactModelDecision::Admitted
}

/// Existing decisions required before transport effects.
pub struct DispatchAdmission {
    pub exact_model: ExactModelDecision,
    pub profile_admitted: bool,
    pub privacy_admitted: bool,
    pub budget_admitted: bool,
    pub authority_admitted: bool,
    pub remote_admitted: bool,
    pub deadline_available: bool,
    pub cancellation_available: bool,
    pub transport_available: bool,
}

/// Pure dispatch result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchDecision {
    Admitted,
    ExactModelDenied,
    PolicyDenied,
    AuthorityDenied,
    RemoteDenied,
    ControlUnavailable,
    MissingTransport,
}

#[must_use]
pub fn admit_dispatch(input: &DispatchAdmission) -> DispatchDecision {
    if input.exact_model != ExactModelDecision::Admitted {
        return DispatchDecision::ExactModelDenied;
    }
    if !input.profile_admitted || !input.privacy_admitted || !input.budget_admitted {
        return DispatchDecision::PolicyDenied;
    }
    if !input.authority_admitted {
        return DispatchDecision::AuthorityDenied;
    }
    if !input.remote_admitted {
        return DispatchDecision::RemoteDenied;
    }
    if !input.deadline_available || !input.cancellation_available {
        return DispatchDecision::ControlUnavailable;
    }
    if !input.transport_available {
        return DispatchDecision::MissingTransport;
    }
    debug_assert_eq!(input.exact_model, ExactModelDecision::Admitted);
    debug_assert!(input.profile_admitted);
    debug_assert!(input.transport_available);
    DispatchDecision::Admitted
}

/// Exact response-model fidelity result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseModelDecision {
    Exact,
    Substituted,
    Missing,
}

#[must_use]
pub fn validate_response_model(expected: &str, observed: Option<&str>) -> ResponseModelDecision {
    match observed {
        None => ResponseModelDecision::Missing,
        Some(value) if value == expected => ResponseModelDecision::Exact,
        Some(_) => ResponseModelDecision::Substituted,
    }
}

/// Safe provider receipt input containing identities only.
pub struct ReceiptInput<'a> {
    pub provider_label: &'a str,
    pub model_label: &'a str,
    pub endpoint_identity: &'a str,
    pub config_identity: &'a str,
    pub catalog_identity: &'a str,
    pub request_identity: &'a str,
    pub response_identity: &'a str,
    pub authority_identity: &'a str,
    pub api_style: ApiStyle,
    pub status: &'a str,
}

/// Metadata-only Mesh receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    pub identity: alloc::string::String,
    pub model_label: alloc::string::String,
    pub status: alloc::string::String,
    pub physical_route: &'static str,
    pub non_claims: [&'static str; RECEIPT_NON_CLAIM_COUNT],
}

/// Receipt leak or unsafe label.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptError {
    UnsafeLabel,
    UnsafeIdentity,
}

/// Projects safe provider evidence without raw provider material.
///
/// # Errors
///
/// Returns [`ReceiptError`] when a label or identity is unsafe.
pub fn project_receipt(input: &ReceiptInput<'_>) -> Result<Receipt, ReceiptError> {
    if !safe_model_id(input.provider_label) || !safe_model_id(input.model_label) || !safe_model_id(input.status) {
        return Err(ReceiptError::UnsafeLabel);
    }
    let identities = [
        input.endpoint_identity,
        input.config_identity,
        input.catalog_identity,
        input.request_identity,
        input.response_identity,
        input.authority_identity,
    ];
    if identities.iter().any(|value| !safe_identity(value)) {
        return Err(ReceiptError::UnsafeIdentity);
    }
    debug_assert!(safe_model_id(input.provider_label));
    debug_assert!(safe_model_id(input.model_label));
    debug_assert!(identities.iter().all(|value| safe_identity(value)));

    let mut hasher = domain_hasher(RECEIPT_DOMAIN);
    for value in identities {
        hash_part(&mut hasher, value);
    }
    hash_part(&mut hasher, input.model_label);
    let identity = hasher.finalize().to_hex().to_string();
    debug_assert!(safe_identity(&identity));
    Ok(Receipt {
        identity,
        model_label: input.model_label.to_string(),
        status: input.status.to_string(),
        physical_route: "unknown",
        non_claims: [
            "not-remote-runtime-integrity",
            "not-peer-honesty",
            "not-physical-route-truth",
            "not-model-correctness",
            "not-split-parity",
            "not-production-readiness",
        ],
    })
}

fn domain_hasher(domain: &str) -> blake3::Hasher {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher
}
fn hash_part(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(&[0]);
    hasher.update(value.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(id: &'static str) -> CatalogRow<'static> {
        CatalogRow {
            id,
            text: Capability::Supported,
            tools: Capability::Supported,
            streaming: Capability::Supported,
            structured_output: Capability::Supported,
            developer_role: Capability::Supported,
            context_tokens: 8_192,
            output_tokens: 1_024,
        }
    }
    fn catalog() -> Catalog {
        normalize_catalog(&CatalogSnapshot {
            endpoint_identity: "endpoint:1",
            config_identity: "config:1",
            freshness_identity: "fresh:1",
            private_gateway: true,
            rows: &[row("model-b"), row("model-a")],
        })
        .unwrap()
    }
    #[test]
    fn catalog_is_sorted_and_deterministic() {
        let value = catalog();
        assert_eq!(value.models[0].id, "model-a");
        assert_eq!(value.identity, catalog().identity);
    }
    #[test]
    fn catalog_rejects_virtual_duplicate_public_and_unknown_bounds() {
        assert_eq!(
            normalize_catalog(&CatalogSnapshot {
                endpoint_identity: "endpoint:1",
                config_identity: "config:1",
                freshness_identity: "fresh:1",
                private_gateway: true,
                rows: &[row("mesh")]
            }),
            Err(CatalogError::UnsafeModelId)
        );
        assert_eq!(
            normalize_catalog(&CatalogSnapshot {
                endpoint_identity: "endpoint:1",
                config_identity: "config:1",
                freshness_identity: "fresh:1",
                private_gateway: true,
                rows: &[row("same"), row("same")]
            }),
            Err(CatalogError::DuplicateModel)
        );
    }
    #[test]
    fn exact_model_denies_virtual_missing_capability_and_substitution() {
        let value = catalog();
        let requirements = Requirements {
            tools: true,
            streaming: true,
            structured_output: false,
            developer_role: false,
            input_tokens: 100,
            output_tokens: 100,
        };
        assert_eq!(admit_exact_model(&value, "model-a", &requirements), ExactModelDecision::Admitted);
        assert_eq!(admit_exact_model(&value, "auto", &requirements), ExactModelDecision::VirtualModelDenied);
        assert_eq!(admit_exact_model(&value, "missing", &requirements), ExactModelDecision::MissingModel);
        assert_eq!(validate_response_model("model-a", Some("model-b")), ResponseModelDecision::Substituted);
    }
    #[test]
    fn authority_precedes_transport() {
        let admitted = DispatchAdmission {
            exact_model: ExactModelDecision::Admitted,
            profile_admitted: true,
            privacy_admitted: true,
            budget_admitted: true,
            authority_admitted: true,
            remote_admitted: true,
            deadline_available: true,
            cancellation_available: true,
            transport_available: true,
        };
        assert_eq!(admit_dispatch(&admitted), DispatchDecision::Admitted);
        assert_eq!(
            admit_dispatch(&DispatchAdmission {
                authority_admitted: false,
                ..admitted
            }),
            DispatchDecision::AuthorityDenied
        );
    }
    #[test]
    fn receipt_is_metadata_only_and_route_unknown() {
        let receipt = project_receipt(&ReceiptInput {
            provider_label: "mesh-private",
            model_label: "model-a",
            endpoint_identity: "endpoint:1",
            config_identity: "config:1",
            catalog_identity: "catalog:1",
            request_identity: "request:1",
            response_identity: "response:1",
            authority_identity: "authority:1",
            api_style: ApiStyle::Responses,
            status: "passed",
        })
        .unwrap();
        assert_eq!(receipt.physical_route, "unknown");
        assert!(!alloc::format!("{receipt:?}").contains("prompt"));
        assert_eq!(
            project_receipt(&ReceiptInput {
                endpoint_identity: "/private/path",
                provider_label: "mesh-private",
                model_label: "model-a",
                config_identity: "config:1",
                catalog_identity: "catalog:1",
                request_identity: "request:1",
                response_identity: "response:1",
                authority_identity: "authority:1",
                api_style: ApiStyle::Responses,
                status: "passed",
            }),
            Err(ReceiptError::UnsafeIdentity)
        );
    }
}
