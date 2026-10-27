//! Thin, inert operation-admission shell for typed authoring.
//!
//! Catalog loading and executable identity observation are explicit actions.
//! Planning uses the existing admission path, never an executor or secret resolver.
use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;

use super::Field;
use super::field;
use crate::engine::Workflow;
use crate::operation::model::ValidatedCatalog;
use crate::operation::model::ValidatedInstance;
use crate::operation::model::ValueType;
const MAX_AUTHORING_JSON_DEPTH: usize = 128;

#[derive(Debug, Clone, Default)]
pub struct Builder {
    catalog_path: String,
    instances_path: String,
    executable_path: String,
    label: String,
    instance_id: String,
    method: String,
    arguments: BTreeMap<String, Value>,
    catalog: Option<ValidatedCatalog>,
    instances: BTreeMap<String, ValidatedInstance>,
}

impl Builder {
    pub fn fields(&self, replace: bool) -> Vec<Field> {
        let mut rows = vec![
            field(
                "operation.catalog_path",
                "Operation catalog path",
                "Typed catalog source; load explicitly after setting both paths",
                &self.catalog_path,
            ),
            field(
                "operation.instances_path",
                "Operation instances path",
                "Data-only instances source; never resolves credentials",
                &self.instances_path,
            ),
            field(
                "operation.load",
                "Load admitted operation choices",
                "Enter load; reads/validates sources only, never runs a process",
                "",
            ),
            field(
                "operation.executable_path",
                "Operation executable identity source",
                "Exact executable to hash at admission; never executed",
                &self.executable_path,
            ),
            field(
                "operation.label",
                "New operation node label",
                "Required for create; replacement preserves the node identity/label",
                &self.label,
            ),
        ];
        if let Some(catalog) = &self.catalog {
            rows.push(field(
                "operation.instance",
                "Select admitted instance",
                &format!("Choose: {}", self.instances.keys().cloned().collect::<Vec<_>>().join(", ")),
                &self.instance_id,
            ));
            if let Some(instance) = self.instances.get(&self.instance_id)
                && let Some(kind) = catalog.types.get(&instance.definition.type_id)
            {
                rows.push(field(
                    "operation.method",
                    "Select admitted method",
                    &format!(
                        "Choose: {}",
                        kind.definition
                            .methods
                            .iter()
                            .map(|method| method.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    &self.method,
                ));
                if let Some(method) = kind.definition.methods.iter().find(|method| method.name == self.method) {
                    for contract in &method.arguments.fields {
                        let id = format!("operation.argument.{}", contract.name);
                        let help = format!(
                            "{:?}; {} bytes max; {}. Collections use member rows; optional remove has a separate action",
                            contract.value_type,
                            contract.max_bytes,
                            if contract.required { "required" } else { "optional" }
                        );
                        let value = self.arguments.get(&contract.name).map(display_value).unwrap_or_default();
                        rows.push(field(&id, &format!("Argument: {}", contract.name), &help, value));
                        if !contract.required {
                            rows.push(field(&format!("operation.argument_remove.{}", contract.name), &format!("Remove argument: {}", contract.name), "Enter remove to omit this optional argument; empty String remains a valid String value", ""));
                        }
                        if let Some(value) = self.arguments.get(&contract.name) {
                            json_rows(&mut rows, &contract.name, "", value);
                        }
                    }
                    rows.push(field("operation.commit", if replace { "Admit and replace selected operation" } else { "Admit and create operation node" }, "Enter admit; revalidates exact catalog/instance/executable and derived plan identities; no execution", ""));
                }
            }
        }
        rows
    }

    pub fn apply(
        &mut self,
        workflow: &mut Workflow,
        node_id: Option<u64>,
        id: &str,
        text: &str,
        runtime: &crate::process::core::RuntimeConfig,
    ) -> Result<(), String> {
        match id {
            "operation.catalog_path" | "operation.instances_path" => {
                let path = required(text)?;
                if id == "operation.catalog_path" {
                    self.catalog_path = path;
                } else {
                    self.instances_path = path;
                }
                self.catalog = None;
                self.instances.clear();
                self.instance_id.clear();
                self.method.clear();
                self.arguments.clear();
            }
            "operation.executable_path" => self.executable_path = required(text)?,
            "operation.label" => self.label = required(text)?,
            "operation.load" => {
                if text != "load" {
                    return Err("Enter load to validate the catalog and instances".into());
                }
                let catalog = crate::operation::format::load_catalog(Path::new(&self.catalog_path))
                    .map_err(|error| error.to_string())?;
                let instances = crate::operation::format::load_instances(Path::new(&self.instances_path), &catalog)
                    .map_err(|error| error.to_string())?;
                self.catalog = Some(catalog);
                self.instances = instances;
                self.instance_id.clear();
                self.method.clear();
                self.arguments.clear();
            }
            "operation.instance" => {
                if !self.instances.contains_key(text) {
                    return Err("Select an admitted instance listed in help".into());
                }
                self.instance_id = text.into();
                self.method.clear();
                self.arguments.clear();
            }
            "operation.method" => {
                let catalog = self.catalog.as_ref().ok_or("Load the catalog first")?;
                let instance = self.instances.get(&self.instance_id).ok_or("Select an instance first")?;
                let kind = catalog.types.get(&instance.definition.type_id).ok_or("Instance type is missing")?;
                if !kind.definition.methods.iter().any(|method| method.name == text) {
                    return Err("Select an admitted method listed in help".into());
                }
                self.method = text.into();
                self.arguments = instance.definition.method_defaults.get(text).cloned().unwrap_or_default();
            }
            "operation.commit" => {
                if text != "admit" {
                    return Err("Enter admit to create or replace the inert operation node".into());
                }
                let expected_catalog = self.catalog.as_ref().ok_or("Load the catalog first")?;
                let expected_instance = self.instances.get(&self.instance_id).ok_or("Select an instance")?;
                let arguments_json = serde_json::to_string(&self.arguments).map_err(|error| error.to_string())?;
                let admitted = crate::operation::cli::admit_for_pack_with_runtime(
                    crate::operation::cli::PlanInput {
                        catalog_path: Path::new(&self.catalog_path),
                        instances_path: Path::new(&self.instances_path),
                        instance_id: &self.instance_id,
                        method: &self.method,
                        arguments_json: &arguments_json,
                        executable_path: Path::new(&self.executable_path),
                    },
                    runtime,
                )
                .map_err(|error| error.to_string())?;
                if &admitted.catalog != expected_catalog || &admitted.instance != expected_instance {
                    return Err("Operation sources changed since selection; reload before admitting".into());
                }
                if let Some(node_id) = node_id.filter(|id| {
                    matches!(workflow.configs.get(id), Some(crate::engine::workflow::NodeConfig::Operation(_)))
                }) {
                    super::node::replace_operation_plan(
                        workflow,
                        node_id,
                        admitted.plan,
                        &admitted.catalog,
                        &admitted.instance,
                    )?;
                } else {
                    super::node::create_operation(
                        workflow,
                        &self.label,
                        admitted.plan,
                        &admitted.catalog,
                        &admitted.instance,
                    )?;
                }
            }
            _ if id.starts_with("operation.argument_remove.") => {
                let name = &id["operation.argument_remove.".len()..];
                if text != "remove" || self.contract(name)?.required {
                    return Err("Only optional arguments can be removed; enter remove".into());
                }
                self.arguments.remove(name);
            }
            _ if id.starts_with("operation.member_add.") => {
                self.apply_json_row(&id["operation.member_add.".len()..], text, true)?
            }
            _ if id.starts_with("operation.argument.") => {
                self.apply_argument(&id["operation.argument.".len()..], text)?
            }
            _ if id.starts_with("operation.value.") => {
                self.apply_json_row(&id["operation.value.".len()..], text, false)?
            }
            _ => return Err("Unknown operation authoring field".into()),
        }
        Ok(())
    }

    fn apply_argument(&mut self, name: &str, text: &str) -> Result<(), String> {
        let contract = self.contract(name)?.clone();
        if text.is_empty() && contract.value_type != ValueType::String {
            if contract.required {
                return Err("A required argument cannot be removed".into());
            }
            self.arguments.remove(name);
            return Ok(());
        }
        let value = match contract.value_type {
            ValueType::String => Value::String(text.into()),
            ValueType::Integer => integer_value(text)?,
            ValueType::Boolean => Value::Bool(text.parse::<bool>().map_err(|_| "Enter true or false")?),
            ValueType::Object if text == "object" => self
                .arguments
                .get(name)
                .filter(|value| value.is_object())
                .cloned()
                .unwrap_or_else(|| Value::Object(Default::default())),
            ValueType::Array if text == "array" => self
                .arguments
                .get(name)
                .filter(|value| value.is_array())
                .cloned()
                .unwrap_or_else(|| Value::Array(Vec::new())),
            ValueType::Object => return Err("Enter object to initialize; then use typed member rows".into()),
            ValueType::Array => return Err("Enter array to initialize; then use typed element rows".into()),
        };
        bounded(&value, contract.max_bytes)?;
        self.arguments.insert(name.into(), value);
        Ok(())
    }

    fn contract(&self, name: &str) -> Result<&crate::operation::model::FieldContract, String> {
        let catalog = self.catalog.as_ref().ok_or("Load the catalog first")?;
        let instance = self.instances.get(&self.instance_id).ok_or("Select an instance")?;
        catalog
            .types
            .get(&instance.definition.type_id)
            .and_then(|kind| kind.definition.methods.iter().find(|method| method.name == self.method))
            .and_then(|method| method.arguments.fields.iter().find(|field| field.name == name))
            .ok_or_else(|| "Unknown argument in the admitted method schema".into())
    }

    fn apply_json_row(&mut self, suffix: &str, text: &str, add: bool) -> Result<(), String> {
        let (name, pointer) = suffix.split_once('|').ok_or("Malformed typed argument row")?;
        let maximum = self.contract(name)?.max_bytes;
        let mut candidate = self.arguments.get(name).cloned().ok_or("Initialize the argument first")?;
        let (parent_pointer, key) = if add {
            (pointer, "")
        } else {
            pointer.rsplit_once('/').ok_or("Malformed member pointer")?
        };
        let parent = candidate.pointer_mut(parent_pointer).ok_or("Stale argument member")?;
        let key = key.replace("~1", "/").replace("~0", "~");
        if add {
            let parts = super::workflow::split_row(text)?;
            if parts.len() != 3 {
                return Err("Use key|type|value with escaped pipe/backslash cells".into());
            }
            let key = &parts[0];
            let value = typed_value(&parts[1], &parts[2])?;
            match parent {
                Value::Object(rows) => {
                    if rows.contains_key(key) {
                        return Err("Object member key is duplicated".into());
                    }
                    rows.insert(key.clone(), value);
                }
                Value::Array(rows) => {
                    if key != "next" {
                        return Err("Use next|type|value to append an array element".into());
                    }
                    rows.push(value);
                }
                _ => return Err("Member owner is not a collection".into()),
            }
        } else {
            match parent {
                Value::Object(rows) => {
                    if !rows.contains_key(&key) {
                        return Err("Stale object member".into());
                    }
                    if text.is_empty() {
                        rows.remove(&key);
                    } else {
                        let value = member_value(rows.get(&key).ok_or("Stale object member")?, text)?;
                        rows.insert(key, value);
                    }
                }
                Value::Array(rows) => {
                    let index = key.parse::<usize>().map_err(|_| "Malformed array index")?;
                    if index >= rows.len() {
                        return Err("Stale array member".into());
                    }
                    if text.is_empty() {
                        rows.remove(index);
                    } else {
                        rows[index] = member_value(&rows[index], text)?;
                    }
                }
                _ => return Err("Member owner is not a collection".into()),
            }
        }
        bounded(&candidate, maximum)?;
        self.arguments.insert(name.into(), candidate);
        Ok(())
    }
}

fn required(value: &str) -> Result<String, String> {
    if value.trim().is_empty() || value.contains('\0') {
        Err("A nonempty value without NUL is required".into())
    } else {
        Ok(value.into())
    }
}

fn bounded(value: &Value, maximum: usize) -> Result<(), String> {
    if !within_depth(value, 0) {
        return Err("Argument exceeds the JSON decoder nesting bound".into());
    }
    serde_json::to_writer(ByteBound { count: 0, maximum }, value).map_err(|error| error.to_string())
}

fn within_depth(value: &Value, depth: usize) -> bool {
    if depth > MAX_AUTHORING_JSON_DEPTH {
        return false;
    }
    match value {
        Value::Object(values) => values.values().all(|value| within_depth(value, depth + 1)),
        Value::Array(values) => values.iter().all(|value| within_depth(value, depth + 1)),
        _ => true,
    }
}

struct ByteBound {
    count: usize,
    maximum: usize,
}
impl std::io::Write for ByteBound {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.count = self
            .count
            .checked_add(bytes.len())
            .filter(|count| *count <= self.maximum)
            .ok_or_else(|| std::io::Error::other("Argument exceeds its admitted byte bound"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn display_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Object(_) => "object".into(),
        Value::Array(_) => "array".into(),
        _ => value.to_string(),
    }
}
fn typed_value(kind: &str, text: &str) -> Result<Value, String> {
    match kind {
        "string" => Ok(Value::String(text.into())),
        "integer" => integer_value(text),
        "boolean" => Ok(Value::Bool(text.parse::<bool>().map_err(|_| "Use true or false")?)),
        "object" if text.is_empty() => Ok(Value::Object(Default::default())),
        "array" if text.is_empty() => Ok(Value::Array(Vec::new())),
        "null" if text.is_empty() || text == "null" => Ok(Value::Null),
        "number" => {
            let number = text.parse::<f64>().map_err(|_| "Invalid finite JSON number")?;
            serde_json::Number::from_f64(number)
                .map(Value::Number)
                .ok_or_else(|| "Invalid finite JSON number".into())
        }
        _ => {
            Err("Use string, integer, number, boolean, null, object, or array; collections need an empty value".into())
        }
    }
}
fn integer_value(text: &str) -> Result<Value, String> {
    if text.starts_with('+') {
        return Err("Use a signed i64 or unsigned u64 integer without +".into());
    }
    text.parse::<i64>()
        .map(Value::from)
        .or_else(|_| text.parse::<u64>().map(Value::from))
        .map_err(|_| "Integer is outside i64/u64 bounds".into())
}
fn member_value(current: &Value, text: &str) -> Result<Value, String> {
    let parts = super::workflow::split_row(text)?;
    if parts.len() != 2 {
        return Err("Use type|value with escaped pipe/backslash cells".into());
    }
    if parts[1].is_empty()
        && ((parts[0] == "object" && current.is_object()) || (parts[0] == "array" && current.is_array()))
    {
        return Ok(current.clone());
    }
    typed_value(&parts[0], &parts[1])
}
fn json_rows(rows: &mut Vec<Field>, name: &str, pointer: &str, value: &Value) {
    match value {
        Value::Object(values) => {
            for (key, value) in values {
                json_member(rows, name, pointer, key, value);
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                json_member(rows, name, pointer, &index.to_string(), value);
            }
        }
        _ => return,
    }
    rows.push(field(&format!("operation.member_add.{name}|{pointer}"), &format!("Add argument member {name}{pointer}"), "object: key|type|value; array: next|type|value; escape pipe/backslash cells; types: string/integer/number/boolean/null/object/array", ""));
}

fn json_member(rows: &mut Vec<Field>, name: &str, pointer: &str, key: &str, value: &Value) {
    let path = format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1"));
    let kind = match value {
        Value::String(_) => "string",
        Value::Number(number) if number.is_i64() || number.is_u64() => "integer",
        Value::Number(_) => "number",
        Value::Bool(_) => "boolean",
        Value::Object(_) => "object",
        Value::Array(_) => "array",
        Value::Null => "null",
    };
    let text = if value.is_object() || value.is_array() {
        super::workflow::row(&[kind, ""])
    } else {
        super::workflow::row(&[kind, &display_value(value)])
    };
    rows.push(field(
        &format!("operation.value.{name}|{path}"),
        &format!("Argument {name}{path}"),
        "type|value edits; blank removes; nested collections have separate member rows",
        text,
    ));
    json_rows(rows, name, &path, value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_values_preserve_numeric_null_empty_and_escaped_collection_members() {
        assert_eq!(typed_value("integer", "18446744073709551615").unwrap(), Value::from(u64::MAX));
        assert_eq!(typed_value("integer", "-9223372036854775808").unwrap(), Value::from(i64::MIN));
        assert_eq!(typed_value("number", "1.25").unwrap(), serde_json::json!(1.25));
        assert_eq!(typed_value("null", "").unwrap(), Value::Null);
        assert_eq!(typed_value("string", "").unwrap(), Value::String(String::new()));
        for (kind, text) in [
            ("integer", "18446744073709551616"),
            ("integer", "-9223372036854775809"),
            ("integer", "1.2"),
            ("integer", "+1"),
            ("number", "NaN"),
            ("number", "inf"),
            ("boolean", "1"),
            ("null", "not-null"),
            ("object", "raw-json"),
        ] {
            assert!(typed_value(kind, text).is_err(), "accepted {kind}={text}");
        }
        let original = serde_json::json!({"a/b": [null,1.25,u64::MAX,{"+":"value|with\\slash"}],"":""});
        let mut rows = Vec::new();
        json_rows(&mut rows, "payload", "", &original);
        let nested = rows.iter().find(|row| row.id == "operation.value.payload|/a~1b/3/+").unwrap();
        assert_eq!(member_value(&Value::Null, &nested.value).unwrap(), "value|with\\slash");
        let collection = rows.iter().find(|row| row.id == "operation.value.payload|/a~1b").unwrap();
        assert_eq!(member_value(&original["a/b"], &collection.value).unwrap(), original["a/b"]);
        assert!(rows.iter().any(|row| row.id == "operation.member_add.payload|/a~1b/3"));
        let empty = rows.iter().find(|row| row.id == "operation.value.payload|/").unwrap();
        assert_eq!(member_value(&Value::Null, &empty.value).unwrap(), "");
    }

    #[test]
    fn collection_depth_and_exact_serialized_byte_bounds_are_enforced() {
        let value = serde_json::json!({"payload":"é"});
        let bytes = serde_json::to_vec(&value).unwrap().len();
        assert!(bounded(&value, bytes).is_ok());
        assert!(bounded(&value, bytes - 1).is_err());
        let mut nested = Value::Null;
        for _ in 0..=MAX_AUTHORING_JSON_DEPTH {
            nested = Value::Array(vec![nested]);
        }
        assert!(bounded(&nested, usize::MAX).unwrap_err().contains("nesting"));
    }
}
