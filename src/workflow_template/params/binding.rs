//! Binding targets: shell-node environment values, never command text. Pure.

/// Check every declared target against the template workflow.
///
/// Each target must name a shell node whose command reads the variable only as the exact
/// double-quoted `"${KEY}"` form. The template must already hold a defaulted parameter's
/// rendered default under that key and must not hold a required parameter's key, so binding
/// the defaults reproduces the template workflow exactly.
pub fn validate_bindings(
    declared: &[super::ParamDeclaration],
    workflow: &crate::engine::Workflow,
) -> Result<(), super::ParamError> {
    for declaration in declared {
        let invalid = || super::ParamError::new(super::ParamErrorCode::InvalidDeclaration, Some(&declaration.name));
        let expected = declaration.default.as_ref().map(super::render);
        for target in &declaration.bindings {
            let config = node_id(workflow, &target.node).and_then(|id| workflow.configs.get(&id));
            let Some(crate::engine::workflow::NodeConfig::Shell { command, env, .. }) = config else {
                return Err(invalid());
            };
            if !references_only_quoted(command, target) {
                return Err(invalid());
            }
            if env.get(&target.shell_env) != expected.as_ref() {
                return Err(invalid());
            }
        }
    }
    Ok(())
}

/// Bind resolved values into a copy of the template workflow as shell environment values.
///
/// Only the declared environment entries change; commands, node kinds, and every other
/// field stay as reviewed, so the set of effect grants cannot widen.
pub fn apply(
    declared: &[super::ParamDeclaration],
    resolved: &super::Resolved,
    template: &crate::engine::Workflow,
) -> Result<crate::engine::Workflow, super::ParamError> {
    let mut workflow = template.clone();
    for declaration in declared {
        let missing = || super::ParamError::new(super::ParamErrorCode::MissingRequired, Some(&declaration.name));
        let rendered = super::render(resolved.get(&declaration.name).ok_or_else(missing)?);
        for target in &declaration.bindings {
            let id = node_id(&workflow, &target.node);
            let config = id.and_then(|id| workflow.configs.get_mut(&id));
            let Some(crate::engine::workflow::NodeConfig::Shell { env, .. }) = config else {
                return Err(super::ParamError::new(super::ParamErrorCode::InvalidDeclaration, Some(&declaration.name)));
            };
            env.insert(target.shell_env.clone(), rendered.clone());
        }
    }
    debug_assert_eq!(workflow.configs.len(), template.configs.len());
    Ok(workflow)
}

/// The identity of the one node with this label, or `None` when no node or several share it.
fn node_id(workflow: &crate::engine::Workflow, label: &str) -> Option<u64> {
    let mut matches = workflow.graph.nodes().filter(|node| node.label == label);
    let node = matches.next()?;
    matches.next().is_none().then_some(node.id.0)
}

/// Whether every expansion of the target's key in `command` is the exact `"${KEY}"` form.
///
/// A bare `$KEY`, an unquoted `${KEY}`, or an operator form such as `${KEY:-x}` is rejected.
/// `$KEY_OTHER` names another variable and is ignored.
fn references_only_quoted(command: &str, target: &super::ParamTarget) -> bool {
    let key = target.shell_env.as_str();
    let bytes = command.as_bytes();
    for (index, _) in command.match_indices(key) {
        let end = index.saturating_add(key.len());
        let is_longer_name = bytes.get(end).is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_');
        let before = &command[..index];
        if before.ends_with("${") {
            let is_quoted =
                before[..before.len().saturating_sub(2)].ends_with('"') && command[end..].starts_with("}\"");
            if !is_quoted {
                return false;
            }
        } else if before.ends_with('$') && !is_longer_name {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    fn accepts(command: &str) -> bool {
        let target = super::super::ParamTarget {
            node: "run tool".to_owned(),
            shell_env: "LATTICE_PARAM_ROOT".to_owned(),
        };
        super::references_only_quoted(command, &target)
    }

    #[test]
    fn only_the_double_quoted_braced_form_is_accepted() {
        assert!(accepts("tool --root \"${LATTICE_PARAM_ROOT}\" --json"));
        assert!(accepts("tool --root ."));
        assert!(accepts("echo \"$LATTICE_PARAM_ROOT_DIR\""));
        assert!(!accepts("tool --root $LATTICE_PARAM_ROOT"));
        assert!(!accepts("tool --root \"$LATTICE_PARAM_ROOT\""));
        assert!(!accepts("tool --root ${LATTICE_PARAM_ROOT}"));
        assert!(!accepts("tool --root \"${LATTICE_PARAM_ROOT:-/}\""));
        assert!(!accepts("tool \"${LATTICE_PARAM_ROOT}\" ${LATTICE_PARAM_ROOT}"));
    }
}
