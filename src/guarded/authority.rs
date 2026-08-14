//! Pure admission checks for identity-bound guarded workflow rounds.

pub const GUARDED_FEATURE_ID: &str = "lattice.guarded-agent-round.v1";
pub const CAIRN_BUNDLE_ID: &str = "cairn-agent-drain";
pub const OCTET_BUNDLE_ID: &str = "octet-agent-burndown";
pub const GUARDED_TOOL_POLICY: &str = "workspace-write-no-push";

const FIXED_ROOT: &str = ".";
const OPERATOR_APPROVER: &str = "operator:local";
const EXPECTED_AGENT_COUNT: usize = 1;
const EXPECTED_MERGE_COUNT: usize = 1;
const EXPECTED_CAIRN_CONDITION_COUNT: usize = 1;
const EXPECTED_OCTET_CONDITION_COUNT: usize = 0;
const EXPECTED_SHELL_COUNT: usize = 4;
const PER_ROUND_AGENT_LIMIT: u64 = 1;
const PER_ROUND_ROUND_LIMIT: u64 = 1;

const CAIRN_COMMANDS: [&str; EXPECTED_SHELL_COUNT] = [
    "lattice guarded cairn-discover --root .",
    "cairn validate --root . --strict",
    "lattice guarded cairn-discover --root .",
    "lattice guarded decide --kind cairn --minimum-delta 1",
];
const OCTET_COMMANDS: [&str; EXPECTED_SHELL_COUNT] = [
    "lattice guarded octet-probe --root . --artifact-dir target/lattice-guarded/octet-before --timeout-secs 600",
    "scripts/check.rs",
    "lattice guarded octet-probe --root . --artifact-dir target/lattice-guarded/octet-after --timeout-secs 600",
    "lattice guarded decide --kind octet --minimum-delta 1",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    UnsupportedBundle,
    InvalidNodeShape,
    InvalidAgent,
    InvalidCommand,
    InvalidAuthority,
    InvalidBudget,
    InvalidTrigger,
    AutomaticRetry,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedBundle => "guarded bundle identity is unsupported",
            Self::InvalidNodeShape => "guarded round node shape is invalid",
            Self::InvalidAgent => "guarded round agent authority is invalid",
            Self::InvalidCommand => "guarded round command set is invalid",
            Self::InvalidAuthority => "guarded orchestration approval authority is invalid",
            Self::InvalidBudget => "guarded orchestration budget is invalid",
            Self::InvalidTrigger => "guarded orchestration trigger policy is invalid",
            Self::AutomaticRetry => "guarded orchestration automatic retry is prohibited",
        })
    }
}

impl std::error::Error for Error {}

struct BundleShape<'a> {
    commands: &'a [&'a str; EXPECTED_SHELL_COUNT],
    prompt_file: &'a str,
    condition_count: usize,
}

#[derive(Default)]
struct WorkflowInventory<'a> {
    shell_commands: Vec<&'a str>,
    agent_count: usize,
    merge_count: usize,
    condition_count: usize,
}

// r[impl lattice.guarded_agent_templates.rounds]
// r[impl lattice.guarded_agent_templates.authority]
pub fn validate_bundle(
    bundle_id: &str,
    workflow: &crate::engine::Workflow,
    orchestration: Option<&crate::orchestration::OrchestrationDefinition>,
) -> Result<(), Error> {
    let shape = bundle_shape(bundle_id)?;
    validate_workflow(workflow, &shape)?;
    validate_orchestration(orchestration.ok_or(Error::InvalidAuthority)?)
}

fn bundle_shape(bundle_id: &str) -> Result<BundleShape<'static>, Error> {
    match bundle_id {
        CAIRN_BUNDLE_ID => Ok(BundleShape {
            commands: &CAIRN_COMMANDS,
            prompt_file: "prompts/cairn-agent-drain.md",
            condition_count: EXPECTED_CAIRN_CONDITION_COUNT,
        }),
        OCTET_BUNDLE_ID => Ok(BundleShape {
            commands: &OCTET_COMMANDS,
            prompt_file: "prompts/octet-agent-burndown.md",
            condition_count: EXPECTED_OCTET_CONDITION_COUNT,
        }),
        _ => Err(Error::UnsupportedBundle),
    }
}

fn validate_workflow(workflow: &crate::engine::Workflow, shape: &BundleShape<'_>) -> Result<(), Error> {
    if !workflow.credentials.is_empty() || !workflow.triggers.is_empty() {
        return Err(Error::InvalidTrigger);
    }
    let mut inventory = WorkflowInventory {
        shell_commands: Vec::with_capacity(EXPECTED_SHELL_COUNT),
        ..WorkflowInventory::default()
    };
    for config in workflow.configs.values() {
        inspect_node(config, shape.prompt_file, &mut inventory)?;
    }
    inventory.shell_commands.sort_unstable();
    let mut expected = shape.commands.to_vec();
    expected.sort_unstable();
    validate_inventory(&inventory, &expected, shape.condition_count)?;
    debug_assert_eq!(inventory.agent_count, EXPECTED_AGENT_COUNT);
    debug_assert_eq!(inventory.shell_commands.len(), EXPECTED_SHELL_COUNT);
    Ok(())
}

fn inspect_node<'a>(
    config: &'a crate::engine::workflow::NodeConfig,
    prompt_file: &str,
    inventory: &mut WorkflowInventory<'a>,
) -> Result<(), Error> {
    let prior_count = inventory
        .shell_commands
        .len()
        .saturating_add(inventory.agent_count)
        .saturating_add(inventory.merge_count)
        .saturating_add(inventory.condition_count);
    match config {
        crate::engine::workflow::NodeConfig::Shell {
            command,
            cwd,
            env,
            timeout_secs,
            ..
        } => {
            validate_shell(cwd.as_deref(), env, *timeout_secs)?;
            inventory.shell_commands.push(command.as_str());
        }
        crate::engine::workflow::NodeConfig::Agent(settings) => {
            inventory.agent_count = inventory.agent_count.saturating_add(1);
            validate_agent(settings, prompt_file)?;
        }
        crate::engine::workflow::NodeConfig::Merge(config) => {
            inventory.merge_count = inventory.merge_count.saturating_add(1);
            validate_merge(config)?;
        }
        crate::engine::workflow::NodeConfig::Condition(config) => {
            inventory.condition_count = inventory.condition_count.saturating_add(1);
            validate_condition(config)?;
        }
        crate::engine::workflow::NodeConfig::Http { .. }
        | crate::engine::workflow::NodeConfig::File { .. }
        | crate::engine::workflow::NodeConfig::ImageInput { .. }
        | crate::engine::workflow::NodeConfig::Subworkflow(_)
        | crate::engine::workflow::NodeConfig::Approval(_) => return Err(Error::InvalidNodeShape),
    }
    let current_count = inventory
        .shell_commands
        .len()
        .saturating_add(inventory.agent_count)
        .saturating_add(inventory.merge_count)
        .saturating_add(inventory.condition_count);
    debug_assert!(current_count >= prior_count);
    debug_assert!(current_count <= prior_count.saturating_add(1));
    Ok(())
}

fn validate_shell(
    cwd: Option<&str>,
    env: &std::collections::HashMap<String, String>,
    timeout_secs: u64,
) -> Result<(), Error> {
    if cwd != Some(FIXED_ROOT) || !env.is_empty() || timeout_secs == 0 {
        return Err(Error::InvalidCommand);
    }
    Ok(())
}

fn validate_agent(settings: &crate::agent_node::AgentNodeSettings, prompt_file: &str) -> Result<(), Error> {
    let has_fixed_shape = settings.target == crate::agent_node::AgentTarget::Pi
        && settings.output_mode == crate::agent_node::AgentOutputMode::Text
        && !settings.prompt.trim().is_empty()
        && settings.prompt_file.as_deref() == Some(prompt_file);
    let has_fixed_authority = settings.model_hint.is_none()
        && settings.tool_policy == GUARDED_TOOL_POLICY
        && settings.cwd.as_deref() == Some(FIXED_ROOT)
        && settings.session.is_none();
    if !has_fixed_shape || !has_fixed_authority {
        return Err(Error::InvalidAgent);
    }
    Ok(())
}

fn validate_merge(config: &crate::control_flow::MergeNodeConfig) -> Result<(), Error> {
    if config.policy != crate::control_flow::MergePolicy::Collect {
        return Err(Error::InvalidNodeShape);
    }
    if config.required_inputs != ["before", "validation", "after"] {
        return Err(Error::InvalidNodeShape);
    }
    Ok(())
}

fn validate_condition(config: &crate::control_flow::ConditionNodeConfig) -> Result<(), Error> {
    if config.branches.len() != EXPECTED_CAIRN_CONDITION_COUNT {
        return Err(Error::InvalidNodeShape);
    }
    let branch = config.branches.first().ok_or(Error::InvalidNodeShape)?;
    let expected_predicate = crate::control_flow::ConditionPredicate::StringContains("\"has_work\":true".to_owned());
    if branch.label != "work" || branch.predicate != expected_predicate {
        return Err(Error::InvalidNodeShape);
    }
    Ok(())
}

fn validate_inventory(
    inventory: &WorkflowInventory<'_>,
    expected_commands: &[&str],
    expected_condition_count: usize,
) -> Result<(), Error> {
    if inventory.shell_commands != expected_commands {
        return Err(Error::InvalidNodeShape);
    }
    if inventory.agent_count != EXPECTED_AGENT_COUNT || inventory.merge_count != EXPECTED_MERGE_COUNT {
        return Err(Error::InvalidNodeShape);
    }
    if inventory.condition_count != expected_condition_count {
        return Err(Error::InvalidNodeShape);
    }
    Ok(())
}

fn validate_orchestration(definition: &crate::orchestration::OrchestrationDefinition) -> Result<(), Error> {
    if definition.completion != crate::orchestration::CompletionPolicy::AwaitProposal {
        return Err(Error::InvalidAuthority);
    }
    if definition.retry.is_some() {
        return Err(Error::AutomaticRetry);
    }
    if !definition.triggers.is_empty() {
        return Err(Error::InvalidTrigger);
    }
    let expected_approvers = std::collections::BTreeSet::from([OPERATOR_APPROVER.to_owned()]);
    if definition.authority.approvers != expected_approvers {
        return Err(Error::InvalidAuthority);
    }
    debug_assert!(definition.retry.is_none());
    debug_assert!(definition.triggers.is_empty());
    validate_budget(definition.budgets)
}

fn validate_budget(policy: crate::orchestration::BudgetPolicy) -> Result<(), Error> {
    let has_required_enforcement = policy.enforced.rounds
        && policy.enforced.agent_invocations
        && policy.enforced.tokens
        && policy.enforced.duration_millis
        && !policy.enforced.monetary_microunits;
    let has_positive_limits = policy.limits.rounds > 0
        && policy.limits.agent_invocations > 0
        && policy.limits.tokens > 0
        && policy.limits.duration_millis > 0;
    let has_per_round_bounds = policy.per_round_upper_bound.rounds == PER_ROUND_ROUND_LIMIT
        && policy.per_round_upper_bound.agent_invocations == PER_ROUND_AGENT_LIMIT
        && policy.per_round_upper_bound.tokens > 0
        && policy.per_round_upper_bound.duration_millis > 0;
    let has_monetary_authority =
        policy.limits.monetary_microunits != 0 || policy.per_round_upper_bound.monetary_microunits != 0;
    if !has_required_enforcement {
        return Err(Error::InvalidBudget);
    }
    if !has_positive_limits {
        return Err(Error::InvalidBudget);
    }
    if !has_per_round_bounds {
        return Err(Error::InvalidBudget);
    }
    if has_monetary_authority {
        return Err(Error::InvalidBudget);
    }
    debug_assert_eq!(policy.per_round_upper_bound.rounds, PER_ROUND_ROUND_LIMIT);
    debug_assert_eq!(policy.per_round_upper_bound.agent_invocations, PER_ROUND_AGENT_LIMIT);
    Ok(())
}
