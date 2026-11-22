//! Binding targets: shell-node environment values, never command text. Pure.

/// Check every declared target against a template workflow.
///
/// The template must hold each optional parameter's rendered default under its key and must
/// not hold a required parameter's key, so binding the defaults reproduces it exactly.
pub fn validate_bindings(
    declared: &[super::ParamDeclaration],
    workflow: &crate::engine::Workflow,
) -> Result<(), super::ParamError> {
    validate_targets(declared, workflow, |declaration| declaration.default.as_ref().map(super::render))
}

/// Check every declared target against a bound workflow: each key must hold its resolved value.
pub fn validate_bound(
    declared: &[super::ParamDeclaration],
    resolved: &super::Resolved,
    workflow: &crate::engine::Workflow,
) -> Result<(), super::ParamError> {
    validate_targets(declared, workflow, |declaration| resolved.get(&declaration.name).map(super::render))
}

/// Shared target checks.
///
/// - No node binds a `LATTICE_PARAM_*` key as a credential, so a credential never shadows a value.
/// - Each target names exactly one shell node, and its key holds `expected`.
/// - Every shell node that receives a value is one simple command in the strict grammar.
/// - Its `cwd` and `env` hold no `{{…}}` expression, so another node's output can never change
///   where or how it looks up its program (for example `PATH={{node:A:…}}`).
fn validate_targets(
    declared: &[super::ParamDeclaration],
    workflow: &crate::engine::Workflow,
    expected: impl Fn(&super::ParamDeclaration) -> Option<String>,
) -> Result<(), super::ParamError> {
    let has_param_credential = workflow
        .credentials
        .values()
        .any(|refs| refs.env.keys().any(|key| key.starts_with(super::PARAM_ENV_PREFIX)));
    if has_param_credential {
        return Err(super::ParamError::new(super::ParamErrorCode::InvalidDeclaration, None));
    }
    let mut node_keys = std::collections::BTreeMap::<u64, Vec<&str>>::new();
    for declaration in declared {
        let invalid = || super::ParamError::new(super::ParamErrorCode::InvalidDeclaration, Some(&declaration.name));
        let value = expected(declaration);
        for target in &declaration.bindings {
            let id = node_id(workflow, &target.node).ok_or_else(invalid)?;
            let Some(crate::engine::workflow::NodeConfig::Shell { env, .. }) = workflow.configs.get(&id) else {
                return Err(invalid());
            };
            if env.get(&target.shell_env) != value.as_ref() {
                return Err(invalid());
            }
            node_keys.entry(id).or_default().push(target.shell_env.as_str());
        }
    }
    for (id, keys) in &node_keys {
        let Some(crate::engine::workflow::NodeConfig::Shell { command, cwd, env, .. }) = workflow.configs.get(id)
        else {
            return Err(super::ParamError::new(super::ParamErrorCode::InvalidDeclaration, None));
        };
        check_receiving_node(command, cwd.as_deref(), env, keys)?;
        let credentials = workflow.credentials.get(id);
        if credentials.is_some_and(|refs| refs.env.keys().any(|key| is_control_key(key))) {
            return Err(super::ParamError::new(super::ParamErrorCode::TargetEnv, None));
        }
    }
    debug_assert!(node_keys.values().all(|keys| !keys.is_empty()));
    debug_assert!(node_keys.len() <= declared.iter().map(|declaration| declaration.bindings.len()).sum::<usize>());
    Ok(())
}

/// Environment keys that make bash source a file, import options, or trace before the command:
///
/// - `BASH_ENV` names a file a non-interactive bash sources;
/// - `SSH_CLIENT`, `SSH2_CLIENT`, and `SHLVL` trigger a `SSH_SOURCE_BASHRC` bash to source
///   `$HOME/.bashrc` before a `-c` command, and `HOME` chooses that file;
/// - `SHELLOPTS` and `BASHOPTS` import options such as `xtrace`;
/// - `PS4` is expanded for every traced command.
///
/// `ENV` is not refused: non-interactive `sh -c` and `bash -c` never read it, and refusing it
/// would block the common `ENV=production`.
const SHELL_CONTROL_KEYS: &[&str] = &[
    "BASH_ENV",
    "SSH_CLIENT",
    "SSH2_CLIENT",
    "SHLVL",
    "HOME",
    "SHELLOPTS",
    "BASHOPTS",
    "PS4",
];

/// Checks for one shell node that receives a value: the strict command grammar, no `{{…}}`
/// expression in `cwd` or `env`, and no static environment that would let the shell run a value.
///
/// The environment check refuses `BASH_FUNC_*` and `%%` keys (exported functions), the shell
/// control keys above, and any value containing `$` or a backtick (expanded when a builtin
/// evaluates the variable arithmetically). Parameter values never contain either character.
fn check_receiving_node(
    command: &str,
    cwd: Option<&str>,
    env: &std::collections::HashMap<String, String>,
    keys: &[&str],
) -> Result<(), super::ParamError> {
    if !super::grammar::is_simple_bound_command(command, keys) {
        return Err(super::ParamError::new(super::ParamErrorCode::InvalidDeclaration, None));
    }
    let has_expression =
        cwd.is_some_and(is_expression) || env.iter().any(|(key, value)| is_expression(key) || is_expression(value));
    if has_expression {
        return Err(super::ParamError::new(super::ParamErrorCode::TargetExpression, None));
    }
    if env.iter().any(is_shell_control) {
        return Err(super::ParamError::new(super::ParamErrorCode::TargetEnv, None));
    }
    debug_assert!(!keys.is_empty());
    debug_assert!(env.keys().all(|key| !key.starts_with("BASH_FUNC_")));
    Ok(())
}

fn is_shell_control((key, value): (&String, &String)) -> bool {
    is_control_key(key) || value.contains(['$', '`'])
}

/// Exported-function keys (`BASH_FUNC_*`, `%%`) and the shell control keys above.
fn is_control_key(key: &str) -> bool {
    key.starts_with("BASH_FUNC_") || key.contains("%%") || SHELL_CONTROL_KEYS.contains(&key)
}

/// Whether text contains a workflow expression marker.
fn is_expression(text: &str) -> bool {
    text.contains("{{")
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
