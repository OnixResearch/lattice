#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "identity material uses the local serialization contract without changing ownership"
)]

use serde::Serialize;

const FRAME_SEPARATOR: &[u8] = b"\0";
const TYPE_REVISION_DOMAIN: &str = "lattice.operation-type-revision.v1";
const METHOD_DOMAIN: &str = "lattice.operation-method.v1";
const SCHEMA_DOMAIN: &str = "lattice.operation-schema.v1";
const CATALOG_DOMAIN: &str = "lattice.operation-catalog.v1";
const INSTANCE_DOMAIN: &str = "lattice.operation-instance.v1";
const ARGUMENTS_DOMAIN: &str = "lattice.operation-arguments.v1";
const INVOCATION_DOMAIN: &str = "lattice.operation-invocation.v1";
const PROMOTION_DOMAIN: &str = "lattice.operation-promotion.v1";
const CONTENT_DOMAIN: &str = "lattice.operation-promotion-content.v1";
const EXECUTOR_DOMAIN: &str = "lattice.operation-executor.v1";
const EXECUTOR_HASH_BUFFER_BYTES: usize = 65_536;
const MAXIMUM_EXECUTOR_BYTES: usize = 268_435_456;
#[cfg(unix)]
const EXECUTABLE_PERMISSION_MASK: u32 = 0o111;

#[derive(Serialize)]
struct TypeRevisionMaterial<'a> {
    id: &'a str,
    schema_version: u32,
    methods: Vec<MethodMaterial<'a>>,
    runtime: &'a super::model::RuntimeRequirement,
    capabilities: &'a [super::model::CapabilityClass],
    credential_bindings: &'a [super::model::CredentialBinding],
    bounds: super::model::Bounds,
    required_non_claims: &'a [String],
}

#[derive(Serialize)]
struct MethodMaterial<'a> {
    name: &'a str,
    arguments: &'a super::model::RecordContract,
    output: &'a super::model::RecordContract,
    preflight: &'a [super::model::PreflightCheck],
    capabilities: &'a [super::model::CapabilityClass],
    side_effect: super::model::SideEffectClass,
}

#[derive(Serialize)]
struct CatalogMaterial<'a> {
    schema: &'a str,
    types: Vec<(&'a str, &'a str)>,
}

#[derive(Serialize)]
struct InstanceMaterial<'a> {
    id: &'a str,
    type_id: &'a str,
    type_revision: &'a str,
    target: &'a std::collections::BTreeMap<String, serde_json::Value>,
    method_defaults: &'a std::collections::BTreeMap<String, std::collections::BTreeMap<String, serde_json::Value>>,
    credentials: &'a std::collections::BTreeMap<String, crate::credentials::SecretRef>,
}

#[derive(Serialize)]
struct InvocationMaterial<'a> {
    catalog_identity: &'a str,
    type_id: &'a str,
    type_revision: &'a str,
    instance_id: &'a str,
    instance_identity: &'a str,
    method: &'a str,
    method_identity: &'a str,
    arguments_identity: &'a str,
    output_contract_identity: &'a str,
    runtime: &'a super::model::RuntimeFact,
    capabilities: &'a [super::model::CapabilityClass],
    target: &'a std::collections::BTreeMap<String, serde_json::Value>,
    credentials: &'a std::collections::BTreeMap<String, crate::credentials::SecretRef>,
    bounds: super::model::Bounds,
    required_non_claims: &'a [String],
}

pub fn validate(value: &str) -> bool {
    let Some(hex) = value.strip_prefix(super::model::BLAKE3_PREFIX) else {
        return false;
    };
    hex.len() == super::model::BLAKE3_HEX_LENGTH
        && hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= super::model::MAXIMUM_ID_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

pub fn hash_bytes(domain: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(FRAME_SEPARATOR);
    hasher.update(bytes);
    format!("{}{}", super::model::BLAKE3_PREFIX, hasher.finalize().to_hex())
}

pub fn hash_value<T: Serialize + ?Sized>(domain: &str, value: &T) -> Result<String, super::Error> {
    let bytes = serde_json::to_vec(value).map_err(|_| super::Error::Encoding)?;
    Ok(hash_bytes(domain, &bytes))
}

pub fn type_revision(definition: &super::model::OperationType) -> Result<String, super::Error> {
    hash_value(TYPE_REVISION_DOMAIN, &TypeRevisionMaterial {
        id: &definition.id,
        schema_version: definition.schema_version,
        methods: definition.methods.iter().map(method_material).collect(),
        runtime: &definition.runtime,
        capabilities: &definition.capabilities,
        credential_bindings: &definition.credential_bindings,
        bounds: definition.bounds,
        required_non_claims: &definition.required_non_claims,
    })
}

pub fn method(type_revision: &str, method: &super::model::Method) -> Result<String, super::Error> {
    hash_value(METHOD_DOMAIN, &(type_revision, method_material(method)))
}

fn method_material(method: &super::model::Method) -> MethodMaterial<'_> {
    MethodMaterial {
        name: &method.name,
        arguments: &method.arguments,
        output: &method.output,
        preflight: &method.preflight,
        capabilities: &method.capabilities,
        side_effect: method.side_effect,
    }
}

pub fn schema(contract: &super::model::RecordContract) -> Result<String, super::Error> {
    hash_value(SCHEMA_DOMAIN, contract)
}

pub fn catalog(catalog: &super::model::Catalog) -> Result<String, super::Error> {
    let types = catalog
        .types
        .iter()
        .map(|definition| (definition.id.as_str(), definition.revision.as_str()))
        .collect();
    hash_value(CATALOG_DOMAIN, &CatalogMaterial {
        schema: &catalog.schema,
        types,
    })
}

pub fn instance(instance: &super::model::OperationInstance) -> Result<String, super::Error> {
    hash_value(INSTANCE_DOMAIN, &InstanceMaterial {
        id: &instance.id,
        type_id: &instance.type_id,
        type_revision: &instance.type_revision,
        target: &instance.target,
        method_defaults: &instance.method_defaults,
        credentials: &instance.credentials,
    })
}

pub fn arguments(arguments: &std::collections::BTreeMap<String, serde_json::Value>) -> Result<String, super::Error> {
    hash_value(ARGUMENTS_DOMAIN, arguments)
}

pub fn invocation(plan: &super::model::InvocationPlan) -> Result<String, super::Error> {
    hash_value(INVOCATION_DOMAIN, &InvocationMaterial {
        catalog_identity: &plan.catalog_identity,
        type_id: &plan.type_id,
        type_revision: &plan.type_revision,
        instance_id: &plan.instance_id,
        instance_identity: &plan.instance_identity,
        method: &plan.method,
        method_identity: &plan.method_identity,
        arguments_identity: &plan.arguments_identity,
        output_contract_identity: &plan.output_contract_identity,
        runtime: &plan.runtime,
        capabilities: &plan.capabilities,
        target: &plan.target,
        credentials: &plan.credentials,
        bounds: plan.bounds,
        required_non_claims: &plan.required_non_claims,
    })
}

#[allow(
    unknown_lints,
    ambiguous_params,
    reason = "the promotion identity frame uses three named and position-stable domain fields"
)]
pub fn promotion(invocation_identity: &str, destination: &str, content_identity: &str) -> Result<String, super::Error> {
    hash_value(PROMOTION_DOMAIN, &(invocation_identity, destination, content_identity))
}

pub fn content(bytes: &[u8]) -> String {
    hash_bytes(CONTENT_DOMAIN, bytes)
}

pub fn executor_file(path: &std::path::Path) -> Result<String, super::Error> {
    use std::io::Read as _;

    if !path.is_absolute() {
        return Err(super::Error::FileAccess);
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|_| super::Error::FileAccess)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(super::Error::FileAccess);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;

        if metadata.permissions().mode() & EXECUTABLE_PERMISSION_MASK == 0 {
            return Err(super::Error::FileAccess);
        }
    }
    if metadata.len() > u64::try_from(MAXIMUM_EXECUTOR_BYTES).map_err(|_| super::Error::FileAccess)? {
        return Err(super::Error::FileAccess);
    }
    let mut file = std::fs::File::open(path).map_err(|_| super::Error::FileAccess)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(EXECUTOR_DOMAIN.as_bytes());
    hasher.update(FRAME_SEPARATOR);
    let mut buffer = [0_u8; EXECUTOR_HASH_BUFFER_BYTES];
    let mut observed = 0_usize;
    loop {
        let count = file.read(&mut buffer).map_err(|_| super::Error::FileAccess)?;
        if count == 0 {
            break;
        }
        observed = observed.checked_add(count).ok_or(super::Error::FileAccess)?;
        if observed > MAXIMUM_EXECUTOR_BYTES {
            return Err(super::Error::FileAccess);
        }
        hasher.update(&buffer[..count]);
    }
    debug_assert!(observed <= MAXIMUM_EXECUTOR_BYTES);
    Ok(format!("{}{}", super::model::BLAKE3_PREFIX, hasher.finalize().to_hex()))
}
