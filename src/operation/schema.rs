#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "schema admission uses its local ordered collection and operation-contract vocabulary"
)]
#![allow(
    assertion_density,
    reason = "schema validation uses typed diagnostics; positive and malformed-value tests cover each invariant"
)]
#![allow(
    unbounded_collection_growth,
    reason = "schema loops apply explicit field-count limits before collection insertion"
)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

const SECRET_FIELD_FRAGMENTS: &[&str] = &[
    "api_key",
    "authorization",
    "credential",
    "password",
    "private_key",
    "secret",
    "token",
];

pub fn validate_contract(contract: &super::model::RecordContract, path: &str) -> Vec<super::model::Diagnostic> {
    let mut diagnostics = Vec::new();
    if contract.fields.len() > super::model::MAXIMUM_SCHEMA_FIELDS {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            path,
            "schema field count exceeds the configured limit",
        ));
    }
    let mut names = BTreeSet::new();
    let mut previous = None;
    for field in contract.fields.iter().take(super::model::MAXIMUM_SCHEMA_FIELDS) {
        let field_path = format!("{path}.{}", field.name);
        if !super::identity::valid_identifier(&field.name) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::InvalidValue,
                &field_path,
                "schema field name is not canonical",
            ));
        }
        if key_is_secret_bearing(&field.name) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::CredentialRejected,
                &field_path,
                "schema field must use an opaque credential reference instead of secret-bearing data",
            ));
        }
        if !names.insert(field.name.as_str()) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::Duplicate,
                &field_path,
                "schema field name is duplicated",
            ));
        }
        if previous.is_some_and(|value: &str| value >= field.name.as_str()) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::InvalidValue,
                path,
                "schema fields must be ordered by canonical name",
            ));
        }
        previous = Some(field.name.as_str());
        if field.max_bytes == 0 || field.max_bytes > super::model::MAXIMUM_VALUE_BYTES {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::Limit,
                &field_path,
                "schema field byte bound is invalid",
            ));
        }
    }
    super::ordered(diagnostics)
}

pub(crate) fn key_is_secret_bearing(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    SECRET_FIELD_FRAGMENTS.iter().any(|fragment| normalized.contains(fragment))
}

pub fn validate_value(
    contract: &super::model::RecordContract,
    value: &serde_json::Value,
    path: &str,
    code: super::model::DiagnosticCode,
) -> Vec<super::model::Diagnostic> {
    validate_value_inner(contract, value, path, code, true)
}

pub fn validate_partial_value(
    contract: &super::model::RecordContract,
    value: &serde_json::Value,
    path: &str,
    code: super::model::DiagnosticCode,
) -> Vec<super::model::Diagnostic> {
    validate_value_inner(contract, value, path, code, false)
}

fn validate_value_inner(
    contract: &super::model::RecordContract,
    value: &serde_json::Value,
    path: &str,
    code: super::model::DiagnosticCode,
    require_required: bool,
) -> Vec<super::model::Diagnostic> {
    let Some(object) = value.as_object() else {
        return vec![super::diagnostic(code, path, "value must be a JSON object")];
    };
    let fields = contract.fields.iter().map(|field| (field.name.as_str(), field)).collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();
    if object.len() > super::model::MAXIMUM_SCHEMA_FIELDS {
        diagnostics.push(super::diagnostic(code, path, "value field count exceeds the admitted limit"));
    }
    for field in contract.fields.iter().take(super::model::MAXIMUM_SCHEMA_FIELDS) {
        let field_path = format!("{path}.{}", field.name);
        match object.get(&field.name) {
            Some(value) => {
                if !matches_type(value, field.value_type) {
                    diagnostics.push(super::diagnostic(code, &field_path, "value has the wrong JSON type"));
                    continue;
                }
                match serde_json::to_vec(value) {
                    Ok(bytes) if bytes.len() <= field.max_bytes => {}
                    Ok(_) => diagnostics.push(super::diagnostic(code, &field_path, "value exceeds its byte bound")),
                    Err(_) => diagnostics.push(super::diagnostic(code, &field_path, "value cannot be encoded")),
                }
            }
            None if require_required && field.required => {
                diagnostics.push(super::diagnostic(code, &field_path, "required value is missing"));
            }
            None => {}
        }
    }
    if contract.deny_unknown_fields {
        for key in object.keys().take(super::model::MAXIMUM_SCHEMA_FIELDS) {
            if !fields.contains_key(key.as_str()) {
                diagnostics.push(super::diagnostic(
                    code,
                    format!("{path}.{key}"),
                    "unknown value is not admitted by the contract",
                ));
            }
        }
    }
    super::ordered(diagnostics)
}

fn matches_type(value: &serde_json::Value, expected: super::model::ValueType) -> bool {
    match expected {
        super::model::ValueType::String => value.is_string(),
        super::model::ValueType::Integer => value.as_i64().is_some() || value.as_u64().is_some(),
        super::model::ValueType::Boolean => value.is_boolean(),
        super::model::ValueType::Object => value.is_object(),
        super::model::ValueType::Array => value.is_array(),
    }
}
