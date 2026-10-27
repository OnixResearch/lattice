#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "instance admission uses its local ordered collections and operation-contract vocabulary"
)]
#![allow(
    assertion_density,
    reason = "fallible instance admission uses typed diagnostics; positive and adversarial tests cover each invariant"
)]
#![allow(
    borrowed_argument_types,
    reason = "instance validation appends diagnostics to one caller-owned growable accumulator"
)]
#![allow(
    unbounded_collection_growth,
    reason = "instance loops apply explicit operation-contract count limits before collection insertion"
)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

const FORBIDDEN_TARGET_KEYS: &[&str] = &[
    "args",
    "capabilities",
    "command",
    "credential",
    "executable",
    "methods",
    "non_claims",
    "program",
    "runtime",
];
pub fn validate(
    instances: super::model::InstanceFile,
    catalog: &super::model::ValidatedCatalog,
) -> Result<BTreeMap<String, super::model::ValidatedInstance>, super::Error> {
    let mut diagnostics = Vec::new();
    if instances.schema != super::model::INSTANCE_SCHEMA {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Schema,
            "instances.schema",
            "instance schema is unsupported",
        ));
    }
    if instances.instances.is_empty() || instances.instances.len() > super::model::MAXIMUM_INSTANCES {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            "instances",
            "instance count is outside the admitted bounds",
        ));
    }
    let mut ids = BTreeSet::new();
    let mut previous = None;
    let mut validated = BTreeMap::new();
    for instance in instances.instances.iter().take(super::model::MAXIMUM_INSTANCES) {
        let path = format!("instances.{}", instance.id);
        let diagnostic_start = diagnostics.len();
        if !ids.insert(instance.id.as_str()) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::Duplicate,
                &path,
                "operation instance id is duplicated",
            ));
        }
        if previous.is_some_and(|value: &str| value >= instance.id.as_str()) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::InvalidValue,
                "instances",
                "operation instances must be ordered by canonical id",
            ));
        }
        previous = Some(instance.id.as_str());
        validate_definition(instance, catalog, &path, &mut diagnostics);
        if diagnostics.len() == diagnostic_start {
            match super::identity::instance(instance) {
                Ok(identity) => {
                    validated.insert(instance.id.clone(), super::model::ValidatedInstance {
                        identity,
                        definition: instance.clone(),
                    });
                }
                Err(_) => diagnostics.push(super::diagnostic(
                    super::model::DiagnosticCode::InvalidIdentity,
                    &path,
                    "operation instance identity could not be encoded",
                )),
            }
        }
    }
    if diagnostics.is_empty() {
        Ok(validated)
    } else {
        Err(super::Error::Diagnostics(super::ordered(diagnostics)))
    }
}

fn validate_definition(
    instance: &super::model::OperationInstance,
    catalog: &super::model::ValidatedCatalog,
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    if !super::identity::valid_identifier(&instance.id) || !super::identity::valid_identifier(&instance.type_id) {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidValue,
            path,
            "operation instance id or type id is not canonical",
        ));
    }
    if instance.display_alias.is_empty() || instance.display_alias.len() > super::model::MAXIMUM_DESCRIPTION_BYTES {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            format!("{path}.display_alias"),
            "operation display alias exceeds its byte bound",
        ));
    }
    let Some(operation_type) = catalog.types.get(&instance.type_id) else {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::MissingType,
            format!("{path}.type_id"),
            "operation instance references an unknown type",
        ));
        return;
    };
    if instance.type_revision != operation_type.definition.revision {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::StaleType,
            format!("{path}.type_revision"),
            "operation instance type revision is stale",
        ));
    }
    if super::runtime::component::runtime_is_component(&operation_type.definition.runtime) {
        if !instance.target.is_empty() {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::InvalidValue,
                format!("{path}.target"),
                "import-free component instances cannot grant host target facts",
            ));
        }
    } else {
        validate_target(instance, path, diagnostics);
        validate_cairn_target(instance, path, diagnostics);
    }
    validate_tags(instance, path, diagnostics);
    validate_defaults(instance, operation_type, path, diagnostics);
    validate_credentials(instance, operation_type, path, diagnostics);
}

fn validate_target(
    instance: &super::model::OperationInstance,
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    if instance.target.is_empty() || instance.target.len() > super::model::MAXIMUM_TARGET_FACTS {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            format!("{path}.target"),
            "operation target fact count is outside the admitted bounds",
        ));
    }
    for (key, value) in instance.target.iter().take(super::model::MAXIMUM_TARGET_FACTS) {
        let key_path = format!("{path}.target.{key}");
        let is_key_forbidden =
            FORBIDDEN_TARGET_KEYS.contains(&key.as_str()) || super::schema::key_is_secret_bearing(key);
        if !super::identity::valid_identifier(key) || is_key_forbidden {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::InvalidValue,
                &key_path,
                "operation target key is unsafe or behavior-bearing",
            ));
        }
        if !matches!(value, serde_json::Value::String(_) | serde_json::Value::Bool(_) | serde_json::Value::Number(_)) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::InvalidValue,
                &key_path,
                "operation target fact must be a scalar JSON value",
            ));
        }
        match serde_json::to_vec(value) {
            Ok(bytes) if bytes.len() <= super::model::MAXIMUM_VALUE_BYTES => {}
            Ok(_) | Err(_) => diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::Limit,
                &key_path,
                "operation target fact exceeds its byte bound",
            )),
        }
    }
}

fn validate_cairn_target(
    instance: &super::model::OperationInstance,
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    let repository_root = instance
        .target
        .get(super::runtime::REPOSITORY_ROOT_TARGET)
        .and_then(serde_json::Value::as_str)
        .map(std::path::Path::new);
    if instance.target.len() != 1 || repository_root.is_none_or(|root| !root.is_absolute()) {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidValue,
            format!("{path}.target"),
            "Cairn operation target must contain only one absolute repository_root string",
        ));
    }
}

fn validate_tags(
    instance: &super::model::OperationInstance,
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    if instance.tags.len() > super::model::MAXIMUM_TAGS {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            format!("{path}.tags"),
            "operation tag count exceeds the admitted limit",
        ));
    }
    for (key, value) in instance.tags.iter().take(super::model::MAXIMUM_TAGS) {
        let is_key_safe = super::identity::valid_identifier(key) && !super::schema::key_is_secret_bearing(key);
        let is_value_safe = !value.is_empty() && value.len() <= super::model::MAXIMUM_DESCRIPTION_BYTES;
        if !is_key_safe || !is_value_safe {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::InvalidValue,
                format!("{path}.tags.{key}"),
                "operation tag is not a bounded safe label",
            ));
        }
    }
}

fn validate_defaults(
    instance: &super::model::OperationInstance,
    operation_type: &super::model::ValidatedType,
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    if instance.method_defaults.len() > super::model::MAXIMUM_DEFAULT_METHODS {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            format!("{path}.method_defaults"),
            "operation method default count exceeds the admitted limit",
        ));
    }
    for (method_name, defaults) in instance.method_defaults.iter().take(super::model::MAXIMUM_DEFAULT_METHODS) {
        let Some(method) = operation_type.definition.methods.iter().find(|method| method.name == *method_name) else {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::MissingMethod,
                format!("{path}.method_defaults.{method_name}"),
                "operation instance defaults reference an unknown method",
            ));
            continue;
        };
        diagnostics.extend(super::schema::validate_partial_value(
            &method.arguments,
            &serde_json::Value::Object(defaults.clone().into_iter().collect()),
            &format!("{path}.method_defaults.{method_name}"),
            super::model::DiagnosticCode::InvalidValue,
        ));
    }
}

fn validate_credentials(
    instance: &super::model::OperationInstance,
    operation_type: &super::model::ValidatedType,
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    if instance.credentials.len() > super::model::MAXIMUM_CREDENTIAL_BINDINGS {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            format!("{path}.credentials"),
            "operation credential count exceeds the admitted limit",
        ));
    }
    for binding in &operation_type.definition.credential_bindings {
        let reference = instance.credentials.get(&binding.name);
        if binding.required && reference.is_none() {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::CredentialRejected,
                format!("{path}.credentials.{}", binding.name),
                "required operation credential reference is missing",
            ));
        }
        if let Some(reference) = reference
            && (crate::credentials::validate_secret_ref(reference).is_err()
                || (!binding.allow_field && reference.field.is_some()))
        {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::CredentialRejected,
                format!("{path}.credentials.{}", binding.name),
                "operation credential reference is malformed or uses a prohibited field",
            ));
        }
    }
    for binding in instance.credentials.keys().take(super::model::MAXIMUM_CREDENTIAL_BINDINGS) {
        if !operation_type.definition.credential_bindings.iter().any(|declared| declared.name == *binding) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::CredentialRejected,
                format!("{path}.credentials.{binding}"),
                "operation instance uses an undeclared credential binding",
            ));
        }
    }
}
