#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "operation formatting uses its local ordered-map, serialization, and metadata vocabulary"
)]
#![allow(
    assertion_density,
    reason = "fallible decoding and inactive metadata projection use typed errors and direct negative tests"
)]

use std::collections::BTreeMap;

use serde::Serialize;

const HOST_TOOL_PROMPT_SNIPPET: &str = "Operation metadata only; execution authority is unavailable.";
const HOST_TOOL_GUIDELINE: &str =
    "Use a reviewed Lattice invocation plan and explicit product authority for execution.";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TypeMetadata {
    pub id: String,
    pub revision: String,
    pub description: String,
    pub methods: Vec<MethodMetadata>,
    pub capabilities: Vec<super::model::CapabilityClass>,
    pub required_non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MethodMetadata {
    pub name: String,
    pub description: String,
    pub arguments_identity: String,
    pub output_identity: String,
    pub capabilities: Vec<super::model::CapabilityClass>,
    pub side_effect: super::model::SideEffectClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InstanceMetadata {
    pub id: String,
    pub identity: String,
    pub display_alias: String,
    pub type_id: String,
    pub type_revision: String,
    pub tag_count: usize,
    pub target_fact_count: usize,
    pub credential_reference_count: usize,
}

pub fn decode_catalog(path: &std::path::Path) -> Result<super::model::Catalog, super::Error> {
    let value = crate::config::load_nickel_as_json(path).map_err(super::Error::Nickel)?;
    serde_json::from_value::<super::model::Catalog>(value).map_err(|_| {
        super::Error::Diagnostics(vec![super::diagnostic(
            super::model::DiagnosticCode::Schema,
            "catalog",
            "operation catalog does not match the strict Rust schema",
        )])
    })
}

pub fn load_catalog(path: &std::path::Path) -> Result<super::model::ValidatedCatalog, super::Error> {
    super::catalog::validate(decode_catalog(path)?)
}

pub fn load_instances(
    path: &std::path::Path,
    catalog: &super::model::ValidatedCatalog,
) -> Result<BTreeMap<String, super::model::ValidatedInstance>, super::Error> {
    let value = crate::config::load_nickel_as_json(path).map_err(super::Error::Nickel)?;
    let instances = serde_json::from_value::<super::model::InstanceFile>(value).map_err(|_| {
        super::Error::Diagnostics(vec![super::diagnostic(
            super::model::DiagnosticCode::Schema,
            "instances",
            "operation instances do not match the strict Rust schema",
        )])
    })?;
    super::instance::validate(instances, catalog)
}

pub fn parse_arguments(input: &str) -> Result<BTreeMap<String, serde_json::Value>, super::Error> {
    if input.len() > super::model::MAXIMUM_OPERATION_BYTES {
        return Err(super::Error::Diagnostics(vec![super::diagnostic(
            super::model::DiagnosticCode::Limit,
            "arguments",
            "operation arguments exceed the admission limit",
        )]));
    }
    let value = serde_json::from_str::<serde_json::Value>(input).map_err(|_| {
        super::Error::Diagnostics(vec![super::diagnostic(
            super::model::DiagnosticCode::InvalidValue,
            "arguments",
            "operation arguments must be valid JSON",
        )])
    })?;
    let serde_json::Value::Object(object) = value else {
        return Err(super::Error::Diagnostics(vec![super::diagnostic(
            super::model::DiagnosticCode::InvalidValue,
            "arguments",
            "operation arguments must be a JSON object",
        )]));
    };
    Ok(object.into_iter().collect())
}

pub fn type_metadata(catalog: &super::model::ValidatedCatalog) -> Vec<TypeMetadata> {
    catalog
        .types
        .values()
        .map(|operation_type| TypeMetadata {
            id: operation_type.definition.id.clone(),
            revision: operation_type.definition.revision.clone(),
            description: operation_type.definition.description.clone(),
            methods: operation_type
                .definition
                .methods
                .iter()
                .map(|method| MethodMetadata {
                    name: method.name.clone(),
                    description: method.description.clone(),
                    arguments_identity: operation_type
                        .argument_schema_identities
                        .get(&method.name)
                        .cloned()
                        .unwrap_or_default(),
                    output_identity: operation_type
                        .output_schema_identities
                        .get(&method.name)
                        .cloned()
                        .unwrap_or_default(),
                    capabilities: method.capabilities.clone(),
                    side_effect: method.side_effect,
                })
                .collect(),
            capabilities: operation_type.definition.capabilities.clone(),
            required_non_claims: operation_type.definition.required_non_claims.clone(),
        })
        .collect()
}

pub fn host_tool_definitions(
    catalog: &super::model::ValidatedCatalog,
) -> Result<Vec<animus_adapters::tool::Definition>, super::Error> {
    let maximum_definition_count = catalog.types.len().saturating_mul(super::model::MAXIMUM_TYPE_METHODS);
    let mut definitions = Vec::with_capacity(maximum_definition_count);
    for operation_type in catalog.types.values() {
        for method in &operation_type.definition.methods {
            let name = format!("operation.{}.{}", operation_type.definition.id, method.name);
            let label = format!("{} {}", operation_type.definition.id, method.name);
            let capability_labels = method.capabilities.iter().map(capability_label).collect::<Vec<_>>();
            let prompt_guidelines = [HOST_TOOL_GUIDELINE];
            let parameter_schema_identity = operation_type
                .argument_schema_identities
                .get(&method.name)
                .map(String::as_str)
                .ok_or(super::Error::Encoding)?;
            let result_metadata_shape = operation_type
                .output_schema_identities
                .get(&method.name)
                .map(String::as_str)
                .ok_or(super::Error::Encoding)?;
            definitions.push(
                animus_adapters::tool::Definition::validate(&animus_adapters::tool::DefinitionDraft {
                    name: Some(&name),
                    label: Some(&label),
                    description: Some(&method.description),
                    parameter_schema_identity: Some(parameter_schema_identity),
                    prompt_snippet: Some(HOST_TOOL_PROMPT_SNIPPET),
                    prompt_guidelines: &prompt_guidelines,
                    source_identity: Some(&operation_type.definition.revision),
                    provenance: Some(&catalog.identity),
                    capability_labels: &capability_labels,
                    result_metadata_shape: Some(result_metadata_shape),
                    status: animus_adapters::tool::DefinitionStatus::Inactive,
                })
                .map_err(|_| super::Error::Invalid("operation host-tool metadata is invalid".to_string()))?,
            );
        }
    }
    Ok(definitions)
}

fn capability_label(capability: &super::model::CapabilityClass) -> &'static str {
    match capability {
        super::model::CapabilityClass::ProcessRead => "process_read",
        super::model::CapabilityClass::CredentialRead => "credential_read",
        super::model::CapabilityClass::FileRead => "file_read",
    }
}

pub fn instance_metadata(instances: &BTreeMap<String, super::model::ValidatedInstance>) -> Vec<InstanceMetadata> {
    instances
        .values()
        .map(|instance| InstanceMetadata {
            id: instance.definition.id.clone(),
            identity: instance.identity.clone(),
            display_alias: instance.definition.display_alias.clone(),
            type_id: instance.definition.type_id.clone(),
            type_revision: instance.definition.type_revision.clone(),
            tag_count: instance.definition.tags.len(),
            target_fact_count: instance.definition.target.len(),
            credential_reference_count: instance.definition.credentials.len(),
        })
        .collect()
}
