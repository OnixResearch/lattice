# Repository-owned agent completion check registry.
# Runtime code consumes only the checked generated JSON export.
# r[impl agent_harness_runtime.authoritative_completion.registry]

let contracts = import "../policy/contracts/primitives.ncl" in

let registry_schema_label = "animus.agent-check-registry.v1" in
let cairn_schema_label = "cairn.agent-completion.v2" in
let repository_identity = "onixresearch/animus" in
let registry_identity = "animus-agent-checks" in
let pinned_cairn_policy_hash = "302bc1f8bcb27920c37a6bdc21865e2ca1fbb5a38c268570b3fc326734e59942" in
let maximum_check_count = 64 in
let maximum_argv_count = 32 in
let maximum_reference_count = 32 in
let capability_classes = ["cairn", "local-process", "nix"] in
let tier_classes = ["quick", "standard", "full"] in
let risk_classes = ["low", "medium", "high"] in
let timeout_classes = ["short", "medium", "long"] in
let output_budget_classes = ["small", "medium", "large"] in
let safe_working_root = "<repository-root>" in

let SafeIdentifier = contracts.safe_identifier "AGENT_CHECK_IDENTIFIER" in
let SafePath = contracts.safe_repository_relative_path "AGENT_CHECK_PATH" in
let CairnPolicyHash = contracts.exact "AGENT_CHECK_CAIRN_POLICY_HASH" pinned_cairn_policy_hash in
let CairnSchema = contracts.exact "AGENT_CHECK_CAIRN_SCHEMA" cairn_schema_label in
let RegistrySchema = contracts.exact "AGENT_CHECK_REGISTRY_SCHEMA" registry_schema_label in
let RepositoryId = contracts.exact "AGENT_CHECK_REPOSITORY" repository_identity in
let RegistryId = contracts.exact "AGENT_CHECK_REGISTRY_ID" registry_identity in
let Capability = contracts.enum "AGENT_CHECK_CAPABILITY" capability_classes in
let Tier = contracts.enum "AGENT_CHECK_TIER" tier_classes in
let Risk = contracts.enum "AGENT_CHECK_RISK" risk_classes in
let Timeout = contracts.enum "AGENT_CHECK_TIMEOUT" timeout_classes in
let OutputBudget = contracts.enum "AGENT_CHECK_OUTPUT_BUDGET" output_budget_classes in
let WorkingRoot = contracts.exact "AGENT_CHECK_WORKING_ROOT" safe_working_root in
let Argv = contracts.stable_validator "AGENT_CHECK_ARGV" (fun value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.is_match "^[A-Za-z0-9._/#=<>-]+$" value
) in

let Reproduction = {
  argv | Array Argv,
  working_root | WorkingRoot,
} in
let CheckRow = {
  check_id | SafeIdentifier,
  profile | SafeIdentifier,
  argv | Array Argv,
  affected_paths | Array SafePath,
  affected_crates | Array SafeIdentifier,
  affected_surfaces | Array SafeIdentifier,
  prerequisites | Array SafeIdentifier,
  capability | Capability,
  tier | Tier,
  risk | Risk,
  timeout_class | Timeout,
  output_budget_class | OutputBudget,
  expected_artifacts | Array SafePath,
  reproduction | Reproduction,
  documentation_refs | Array SafePath,
  positive_contract | SafePath,
  negative_contract | SafePath,
} in
let RegistryShape = {
  schema | RegistrySchema,
  repository_id | RepositoryId,
  registry_id | RegistryId,
  supported_cairn_schema | CairnSchema,
  cairn_policy_hash | CairnPolicyHash,
  fallback_check_ids | Array SafeIdentifier,
  checks | Array CheckRow,
} in

let _argv_is_bounded = fun argv =>
  std.array.length argv > 0
  && std.array.length argv <= maximum_argv_count
in
let _references_are_bounded = fun values =>
  std.array.length values > 0
  && std.array.length values <= maximum_reference_count
in
let _row_is_complete = fun row =>
  _argv_is_bounded row.argv
  && row.reproduction.argv == row.argv
  && _references_are_bounded row.affected_paths
  && _references_are_bounded row.affected_surfaces
  && _references_are_bounded row.expected_artifacts
  && _references_are_bounded row.documentation_refs
  && contracts.all_unique row.argv
  && contracts.all_unique row.affected_paths
  && contracts.all_unique row.affected_crates
  && contracts.all_unique row.affected_surfaces
  && contracts.all_unique row.prerequisites
  && contracts.all_unique row.expected_artifacts
  && contracts.all_unique row.documentation_refs
in
let _known_id = fun rows value =>
  std.array.any (fun row => row.check_id == value) rows
in
let _prerequisites_are_known = fun rows row =>
  std.array.all
    (fun prerequisite => prerequisite != row.check_id && _known_id rows prerequisite)
    row.prerequisites
in
let _registry_validator = fun registry =>
  if std.array.length registry.checks == 0 || std.array.length registry.checks > maximum_check_count then
    'Error { message = "AGENT_CHECK_COUNT" }
  else if !contracts.unique_by (fun row => row.check_id) registry.checks then
    'Error { message = "AGENT_CHECK_DUPLICATE_ID" }
  else if std.array.length registry.fallback_check_ids == 0 || !contracts.all_unique registry.fallback_check_ids then
    'Error { message = "AGENT_CHECK_FALLBACK_SET" }
  else if !std.array.all (fun check_id => _known_id registry.checks check_id) registry.fallback_check_ids then
    'Error { message = "AGENT_CHECK_UNKNOWN_FALLBACK" }
  else if !std.array.all _row_is_complete registry.checks then
    'Error { message = "AGENT_CHECK_INCOMPLETE_ROW" }
  else if !std.array.all (fun row => _prerequisites_are_known registry.checks row) registry.checks then
    'Error { message = "AGENT_CHECK_UNKNOWN_PREREQUISITE" }
  else
    'Ok
in
let AgentCheckRegistry = contracts.with_validator RegistryShape _registry_validator in

let registry_value = {
  schema = registry_schema_label,
  repository_id = repository_identity,
  registry_id = registry_identity,
  supported_cairn_schema = cairn_schema_label,
  cairn_policy_hash = pinned_cairn_policy_hash,
  fallback_check_ids = ["nickel-config", "rust-workspace-tests", "cairn-validate"],
  checks = [
    {
      check_id = "nickel-config",
      profile = "nickel-config",
      argv = ["nix", "run", ".#check-nickel-configs"],
      affected_paths = ["config", "policy", "Cargo.toml", "flake.nix"],
      affected_crates = [],
      affected_surfaces = ["configuration", "generated-policy"],
      prerequisites = [],
      capability = "nix",
      tier = "quick",
      risk = "low",
      timeout_class = "short",
      output_budget_class = "small",
      expected_artifacts = ["config/generated"],
      reproduction = { argv = ["nix", "run", ".#check-nickel-configs"], working_root = safe_working_root },
      documentation_refs = ["README.md"],
      positive_contract = "config/agent-check-registry/fixtures/valid-baseline.ncl",
      negative_contract = "config/agent-check-registry/fixtures/negative-unsafe-argv.ncl",
    },
    {
      check_id = "steel-runtime-policy",
      profile = "steel-runtime-policy",
      argv = ["scripts/cairn/check-steel-runtime-policy", "--root", "."],
      affected_paths = ["policy/steel-runtime", "crates/animus-steel", "crates/animus-runtime"],
      affected_crates = ["animus-steel", "animus-runtime"],
      affected_surfaces = ["steel-runtime", "completion-candidate"],
      prerequisites = ["nickel-config"],
      capability = "local-process",
      tier = "quick",
      risk = "medium",
      timeout_class = "medium",
      output_budget_class = "medium",
      expected_artifacts = ["target/cairn/focused-rails/steel-runtime-policy.md"],
      reproduction = { argv = ["scripts/cairn/check-steel-runtime-policy", "--root", "."], working_root = safe_working_root },
      documentation_refs = ["README.md"],
      positive_contract = "crates/animus-steel/src/policy/tests.rs",
      negative_contract = "crates/animus-steel/src/action.rs",
    },
    {
      check_id = "rust-workspace-tests",
      profile = "rust-workspace-tests",
      argv = ["cargo", "test", "--workspace", "--locked"],
      affected_paths = ["crates", "Cargo.toml", "Cargo.lock"],
      affected_crates = ["animus-runtime"],
      affected_surfaces = ["rust", "runtime"],
      prerequisites = [],
      capability = "local-process",
      tier = "standard",
      risk = "medium",
      timeout_class = "long",
      output_budget_class = "large",
      expected_artifacts = ["target"],
      reproduction = { argv = ["cargo", "test", "--workspace", "--locked"], working_root = safe_working_root },
      documentation_refs = ["README.md"],
      positive_contract = "crates/animus-runtime/src/authoritativecompletion/tests.rs",
      negative_contract = "crates/animus-runtime/src/authoritativecompletion/tests.rs",
    },
    {
      check_id = "rust-workspace-clippy",
      profile = "rust-workspace-clippy",
      argv = ["cargo", "clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"],
      affected_paths = ["crates", "Cargo.toml", "Cargo.lock"],
      affected_crates = ["animus-runtime"],
      affected_surfaces = ["rust", "lint"],
      prerequisites = ["rust-workspace-tests"],
      capability = "local-process",
      tier = "standard",
      risk = "medium",
      timeout_class = "long",
      output_budget_class = "large",
      expected_artifacts = ["target"],
      reproduction = { argv = ["cargo", "clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"], working_root = safe_working_root },
      documentation_refs = ["README.md"],
      positive_contract = "crates/animus-runtime/src/authoritativecompletion/tests.rs",
      negative_contract = "crates/animus-runtime/src/authoritativecompletion/tests.rs",
    },
    {
      check_id = "tigerstyle",
      profile = "tigerstyle",
      argv = ["cargo", "tigerstyle", "check"],
      affected_paths = ["crates", "scripts"],
      affected_crates = ["animus-runtime"],
      affected_surfaces = ["rust", "scripts", "tigerstyle"],
      prerequisites = ["rust-workspace-clippy"],
      capability = "local-process",
      tier = "standard",
      risk = "high",
      timeout_class = "long",
      output_budget_class = "large",
      expected_artifacts = ["artifacts"],
      reproduction = { argv = ["cargo", "tigerstyle", "check"], working_root = safe_working_root },
      documentation_refs = ["AGENTS.md"],
      positive_contract = "crates/animus-runtime/src/authoritativecompletion/tests.rs",
      negative_contract = "crates/animus-runtime/src/authoritativecompletion/tests.rs",
    },
    {
      check_id = "cairn-validate",
      profile = "cairn-validate",
      argv = ["nix", "run", ".#cairn", "--", "validate", "--root", "."],
      affected_paths = ["cairn", "docs", "config", "scripts", "flake.nix"],
      affected_crates = [],
      affected_surfaces = ["lifecycle", "documentation", "configuration"],
      prerequisites = ["nickel-config"],
      capability = "cairn",
      tier = "quick",
      risk = "high",
      timeout_class = "medium",
      output_budget_class = "medium",
      expected_artifacts = ["cairn"],
      reproduction = { argv = ["nix", "run", ".#cairn", "--", "validate", "--root", "."], working_root = safe_working_root },
      documentation_refs = ["docs/request-lifecycle.md"],
      positive_contract = ".cairn/specs/agent-harness-runtime/spec.md",
      negative_contract = ".cairn/specs/agent-harness-runtime/spec.md",
    },
    {
      check_id = "authoritative-completion-rail",
      profile = "authoritative-completion-rail",
      argv = ["scripts/cairn/check-authoritative-agent-completion", "--root", "."],
      affected_paths = ["config/agent-check-registry.ncl", "config/generated/agent-check-registry.json", "crates/animus-adapters", "crates/animus-runtime", "crates/animus-std", "crates/animus-steel", "policy/steel-runtime", "scripts/cairn/check-authoritative-agent-completion"],
      affected_crates = ["animus-adapters", "animus-runtime", "animus-std", "animus-steel"],
      affected_surfaces = ["authoritative-completion", "completion-candidate", "validation"],
      prerequisites = ["nickel-config", "steel-runtime-policy", "rust-workspace-tests"],
      capability = "cairn",
      tier = "quick",
      risk = "high",
      timeout_class = "long",
      output_budget_class = "large",
      expected_artifacts = ["target/cairn/focused-rails/authoritative-completion.md"],
      reproduction = { argv = ["scripts/cairn/check-authoritative-agent-completion", "--root", "."], working_root = safe_working_root },
      documentation_refs = ["docs/authoritative-agent-completion.md"],
      positive_contract = "crates/animus-runtime/src/authoritativecompletion/tests.rs",
      negative_contract = "crates/animus-runtime/src/authoritativecompletion/tests.rs",
    },
  ],
} in
{
  agent_check_registry_contract = AgentCheckRegistry,
  agent_check_registry | AgentCheckRegistry = registry_value,
}
