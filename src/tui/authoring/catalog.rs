//! Checked parity inventory for the current portable contract.
//!
//! Evidence is behavioral: the owner regression tables and TUI/store/Nickel
//! fixtures exercise these actions. This catalog does not infer parity from a
//! value merely surviving serialization.
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Typed,
    AdmittedPlan,
    AllocatedIdentity,
    RejectedPlaintext,
    DisplayOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    pub id: String,
    pub portable: Vec<String>,
    pub mode: Mode,
    pub action: &'static str,
    pub reset: &'static str,
    pub evidence: &'static str,
    pub spec: super::metadata::Spec,
}

// r[impl lattice.tui_ux.portable_authoring_parity.catalog]
pub fn capabilities() -> Vec<Capability> {
    let mut rows = Vec::new();
    for (provider, id) in crate::node_kind_provider::authoring_capabilities() {
        let admitted = provider.kind() == crate::node_kind_provider::NodeKind::Operation;
        rows.push(Capability {
            id: id.into(),
            portable: vec![node_path(id)],
            mode: if admitted { Mode::AdmittedPlan } else { Mode::Typed },
            action: if admitted {
                "Ctrl-W / operation / catalog-instance-method-typed-arguments / admit"
            } else {
                "Ctrl-W / config / typed scalar or member row"
            },
            reset: if admitted {
                "admit replacement plan or remove owner; no free-text derived identities"
            } else {
                "blank optional/member; required fields use validated replacement or owner removal"
            },
            evidence: "tui::authoring::node::tests; tui::authoring::operation::tests; tui::authoring::tests",
            spec: super::metadata::spec(id),
        });
    }
    for id in super::workflow::capability_ids() {
        rows.push(Capability {
            id: id.into(),
            portable: workflow_paths(id),
            mode: Mode::Typed,
            action: "Ctrl-W / owner / validated typed row",
            reset: "blank optional/member or explicit remove/reset action",
            evidence: "tui::authoring::workflow::tests; tui::authoring::tests",
            spec: super::metadata::spec(id),
        });
    }
    for (id, paths) in [
        ("identity.node", vec!["nodes[].id", "configs[].node_id"]),
        ("identity.kind", vec!["configs[].kind"]),
    ] {
        rows.push(Capability {
            id: id.into(),
            portable: paths.into_iter().map(str::to_string).collect(),
            mode: Mode::AllocatedIdentity,
            action: "guided node creation; aggregate deletion; accepted-plan replacement preserves owner",
            reset: "remove owner; IDs are checked, not implicitly reused",
            evidence: "tui_aggregate_lifecycle; tui::authoring::tests",
            spec: super::metadata::spec(id),
        });
    }
    rows.push(Capability {
        id: "credential.plaintext_value".into(),
        portable: vec!["configs[].secret_env[].value".into()],
        mode: Mode::RejectedPlaintext,
        action: "no plaintext authoring: opaque provider/key/field references only",
        reset: "remove reference",
        evidence: "test_credentials; test_workflow_format",
        spec: super::metadata::spec("credential.plaintext_value"),
    });
    rows.sort_by(|left, right| left.id.cmp(&right.id));
    rows
}

fn node_path(id: &str) -> String {
    let suffix = id.strip_prefix("config.").expect("node capability owner");
    let (kind, field) = suffix.split_once('.').expect("node capability field");
    let field = match kind {
        "shell" => field.into(),
        "http" if field == "method" => "http_method".into(),
        "http" => field.into(),
        "file" if ["operation", "path", "content"].contains(&field) => format!("file_{field}"),
        "file" => field.into(),
        "image_input" => format!("image_{field}"),
        "agent" => format!("agent_{field}"),
        "condition" => format!("condition_{field}"),
        "merge" => format!("merge_{field}"),
        "subworkflow" if field == "workflow_name" => "subworkflow_name".into(),
        "subworkflow" => format!("subworkflow_{field}"),
        "approval" => format!("approval[].{field}"),
        "operation" => format!("operation_plan[].{field}"),
        _ => format!("unknown:{kind}.{field}"),
    };
    format!("configs[].{field}")
}

fn workflow_paths(id: &str) -> Vec<String> {
    let path = if let Some(field) = id.strip_prefix("workflow.") {
        format!("meta.{field}")
    } else if let Some(field) = id.strip_prefix("runner.") {
        format!("configs[].{field}")
    } else if let Some(field) = id.strip_prefix("credential.env.") {
        format!("configs[].secret_env[].{field}")
    } else if let Some(field) = id.strip_prefix("graph.port.") {
        if field == "direction" {
            return vec!["nodes[].input_ports[]".into(), "nodes[].output_ports[]".into()];
        }
        return vec![
            format!("nodes[].input_ports[].{field}"),
            format!("nodes[].output_ports[].{field}"),
        ];
    } else if let Some(field) = id.strip_prefix("graph.edge.") {
        format!("edges[].{field}")
    } else if let Some(field) = id.strip_prefix("graph.") {
        format!("nodes[].{field}")
    } else if let Some(field) = id.strip_prefix("trigger.") {
        format!("triggers[].{}", field.replace("input_bindings.", "input_bindings[]."))
    } else {
        format!("unknown:{id}")
    };
    vec![path]
}

/// Reject omitted/duplicated/stale/display-only capabilities and portable shape
/// additions. Callers supply the serialized current fixture, not a copied schema.
// r[impl lattice.tui_ux.portable_authoring_parity.catalog.scenario.missing]
pub fn check(rows: &[Capability], portable: &serde_json::Value) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let mut ids = BTreeSet::new();
    let expected = capabilities().into_iter().map(|row| row.id).collect::<BTreeSet<_>>();
    for row in rows {
        if !ids.insert(row.id.clone()) {
            errors.push(format!("duplicate capability {}", row.id));
        }
        if !expected.contains(&row.id) {
            errors.push(format!("unknown capability {}", row.id));
        }
        if row.mode == Mode::DisplayOnly {
            errors.push(format!("display-only capability {}", row.id));
        }
        if row.portable.is_empty() || row.action.is_empty() || row.evidence.is_empty() {
            errors.push(format!("incomplete capability {}", row.id));
        }
    }
    for missing in expected.difference(&ids) {
        errors.push(format!("missing capability {missing}"));
    }
    let known = rows.iter().flat_map(|row| row.portable.iter()).cloned().collect::<BTreeSet<_>>();
    inspect_shape(portable, "", &known, &mut errors);
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

fn inspect_shape(value: &serde_json::Value, prefix: &str, known: &BTreeSet<String>, errors: &mut Vec<String>) {
    // Target, argument maps and credential map keys are data owned by admitted
    // operation planning, not fields whose arbitrary keys expand the schema.
    let map = format!("{prefix}{{}}");
    if known.contains(&map) {
        return;
    }
    let child_map = format!("{map}.");
    if known.iter().any(|path| path.starts_with(&child_map)) {
        if let Some(values) = value.as_object() {
            for value in values.values() {
                inspect_shape(value, &map, known, errors);
            }
        }
        return;
    }
    match value {
        serde_json::Value::Object(values) => {
            for (key, value) in values {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                let array_path = format!("{path}[]");
                let covered = known.contains(&path)
                    || known.contains(&array_path)
                    || known.iter().any(|known| {
                        known.starts_with(&format!("{path}."))
                            || known.starts_with(&format!("{array_path}."))
                            || known.starts_with(&format!("{path}{{}}"))
                    });
                if !covered {
                    errors.push(format!("uncovered portable field {path}"));
                }
                inspect_shape(value, &path, known, errors);
            }
        }
        serde_json::Value::Array(values) => {
            let path = format!("{prefix}[]");
            for value in values {
                inspect_shape(value, &path, known, errors);
            }
        }
        _ => {}
    }
}
