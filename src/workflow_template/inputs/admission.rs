//! Admit raw inputs for one saved workflow against its retained template. Pure.

// r[impl lattice.workflow_templates.run_inputs]
/// Type, check, screen, and bind raw inputs for a saved workflow.
///
/// The saved workflow must still have the exact revision retained when it was saved from a
/// bundle, and the catalog must still hold that exact template identity. Inputs are typed by
/// its declarations, resolved by `params::resolve`, screened by [`super::screen`], and bound by
/// [`super::super::bind`], which repeats full bundle admission. Any failure happens before a
/// caller reserves a run, writes a record, or runs a node.
pub fn admit(
    facts: super::SavedFacts<'_>,
    raw: &std::collections::BTreeMap<String, super::RawInput>,
) -> Result<super::Admitted, super::InputError> {
    let saved = facts.saved.ok_or(super::InputError::WorkflowNotFound)?;
    let binding = facts.binding.ok_or(super::InputError::NotParameterized)?;
    let saved_revision =
        crate::orchestration::workflow_revision(saved).map_err(|_| super::InputError::SavedWorkflowDrift)?;
    if saved_revision != binding.workflow || saved.meta.name != facts.workflow_name {
        return Err(super::InputError::SavedWorkflowDrift);
    }
    let template = facts
        .catalog
        .iter()
        .find(|bundle| bundle.manifest().id == binding.bundle && bundle.identity() == &binding.template)
        .ok_or(super::InputError::TemplateUnavailable)?;
    let declared = template.manifest().params();
    if declared.is_empty() {
        return Err(super::InputError::NotParameterized);
    }
    if has_approval_gate(template.workflow()) {
        return Err(super::InputError::ApprovalGate);
    }
    let supplied = typed_values(declared, raw)?;
    let resolved = super::super::params::resolve(declared, &supplied).map_err(super::InputError::Param)?;
    super::screen(declared, template.workflow(), &supplied)?;
    let bundle = super::super::bind(template, &supplied).map_err(super::InputError::Bind)?;
    if bundle.workflow().meta.name != facts.workflow_name {
        return Err(super::InputError::SavedWorkflowDrift);
    }
    let identity = input_identity(&bundle, &resolved, declared, &supplied);
    debug_assert_eq!(identity.params.len(), declared.len());
    debug_assert!(bundle.params().is_some_and(|bound| bound.resolved == resolved));
    Ok(super::Admitted { bundle, identity })
}

// r[impl lattice.workflow_templates.run_inputs.start]
/// Re-verify admitted inputs at the run start boundary, before a run id is reserved.
///
/// The bound bundle must pass the same `reverify` as save and start, refusing drifted bound
/// content with `bound_content_drift`, and the recorded input identity must name exactly that
/// bundle and revision.
pub fn reverify_admitted(admitted: &super::Admitted) -> Result<(), super::InputError> {
    let bundle = &admitted.bundle;
    super::super::reverify(bundle).map_err(super::InputError::Bind)?;
    let identity = &admitted.identity;
    let bound = bundle.params().ok_or(super::InputError::Bind(super::super::model::Diagnostic {
        bundle: identity.bundle.clone(),
        code: super::super::model::DiagnosticCode::ParamsUnbound,
        message: "bundle parameters are not bound",
    }))?;
    let identity_checks = [
        identity.bundle == bundle.manifest().id,
        identity.template == bound.template,
        identity.bound == *bundle.identity(),
        identity.workflow == *bundle.workflow_revision(),
        identity.values == *bound.resolved.identity(),
    ];
    if identity_checks.contains(&false) {
        return Err(super::InputError::Bind(super::super::model::Diagnostic {
            bundle: bundle.manifest().id.clone(),
            code: super::super::model::DiagnosticCode::BoundContentDrift,
            message: "recorded input identity does not match the bound bundle",
        }));
    }
    debug_assert_eq!(identity.workflow.workflow_name, bundle.workflow().meta.name);
    Ok(())
}

fn input_identity(
    bundle: &super::super::model::ValidatedBundle,
    resolved: &super::super::params::Resolved,
    declared: &[super::super::params::ParamDeclaration],
    supplied: &std::collections::BTreeMap<String, super::super::params::ParamValue>,
) -> super::InputIdentity {
    let mut params = declared
        .iter()
        .map(|declaration| super::InputName {
            name: declaration.name.clone(),
            kind: super::InputKind::of(&declaration.value_type),
            supplied: supplied.contains_key(&declaration.name),
        })
        .collect::<Vec<_>>();
    params.sort_by(|left, right| left.name.cmp(&right.name));
    debug_assert!(params.windows(2).all(|pair| pair[0].name < pair[1].name));
    let template = bundle.params().map_or_else(|| bundle.identity().clone(), |bound| bound.template.clone());
    debug_assert_ne!(&template, bundle.identity());
    super::InputIdentity {
        bundle: bundle.manifest().id.clone(),
        template,
        bound: bundle.identity().clone(),
        workflow: bundle.workflow_revision().clone(),
        values: resolved.identity().clone(),
        params,
    }
}

/// Type each raw value by its declaration. An undeclared name is refused without echoing it.
fn typed_values(
    declared: &[super::super::params::ParamDeclaration],
    raw: &std::collections::BTreeMap<String, super::RawInput>,
) -> Result<std::collections::BTreeMap<String, super::super::params::ParamValue>, super::InputError> {
    let typed = raw
        .iter()
        .map(|(name, value)| {
            let declaration = declared.iter().find(|declaration| &declaration.name == name).ok_or(
                super::InputError::Param(super::super::params::ParamError {
                    code: super::super::params::ParamErrorCode::UnknownParam,
                    param: None,
                }),
            )?;
            let typed_value = coerce(&declaration.value_type, value).ok_or_else(|| {
                super::InputError::Param(super::super::params::ParamError {
                    code: super::super::params::ParamErrorCode::WrongType,
                    param: Some(declaration.name.clone()),
                })
            })?;
            Ok((name.clone(), typed_value))
        })
        .collect::<Result<std::collections::BTreeMap<_, _>, super::InputError>>()?;
    debug_assert_eq!(typed.len(), raw.len());
    Ok(typed)
}

/// A `--input` text takes its declared type; a JSON value keeps its JSON type, so `"true"` for a
/// `bool` is a wrong type.
fn coerce(
    value_type: &super::super::params::ParamType,
    raw: &super::RawInput,
) -> Option<super::super::params::ParamValue> {
    match raw {
        super::RawInput::Text(text) => text_value(value_type, text),
        super::RawInput::Json(value) => json_value(value),
    }
}

/// Exactly `true` or `false` for a `bool`, a canonical decimal for an `integer`, and the text
/// for an `enum` or `relative_path`.
fn text_value(value_type: &super::super::params::ParamType, text: &str) -> Option<super::super::params::ParamValue> {
    match value_type {
        super::super::params::ParamType::Bool {} => match text {
            "true" => Some(super::super::params::ParamValue::Bool(true)),
            "false" => Some(super::super::params::ParamValue::Bool(false)),
            _ => None,
        },
        super::super::params::ParamType::Integer { .. } => text
            .parse::<i64>()
            .ok()
            .filter(|value| value.to_string() == text)
            .map(super::super::params::ParamValue::Integer),
        super::super::params::ParamType::Enum { .. } | super::super::params::ParamType::RelativePath { .. } => {
            Some(super::super::params::ParamValue::Text(text.to_owned()))
        }
    }
}

fn json_value(value: &serde_json::Value) -> Option<super::super::params::ParamValue> {
    match value {
        serde_json::Value::Bool(value) => Some(super::super::params::ParamValue::Bool(*value)),
        serde_json::Value::Number(number) => number.as_i64().map(super::super::params::ParamValue::Integer),
        serde_json::Value::String(text) => Some(super::super::params::ParamValue::Text(text.clone())),
        serde_json::Value::Null | serde_json::Value::Array(_) | serde_json::Value::Object(_) => None,
    }
}

/// Whether the workflow has an approval gate. Resume reloads the saved workflow revision, so a
/// run bound to inputs could not resume; such workflows refuse inputs.
pub fn has_approval_gate(workflow: &crate::engine::Workflow) -> bool {
    workflow
        .configs
        .values()
        .any(|config| matches!(config, crate::engine::workflow::NodeConfig::Approval(_)))
}
