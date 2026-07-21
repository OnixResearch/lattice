let CommandExample = {
  argv | Array String,
} in
let ExecPolicyRule = {
  id | String,
  decision | String,
  rationale | String,
  command_prefix | Array String,
  positive_examples | Array CommandExample,
  negative_examples | Array CommandExample,
} in
let InvalidRuleFixture = {
  name | String,
  expected_error | String,
  rule | ExecPolicyRule,
} in
let SandboxContext = {
  active_sandbox | String,
  requested_capability | String,
  escalation_reason | String,
} in
let ExecPolicyCatalog = {
  schema | String,
  sandbox_context | SandboxContext,
  valid_rules | Array ExecPolicyRule,
  invalid_rule_fixtures | Array InvalidRuleFixture,
} in
let policy_schema = "animus.exec-policy.v1" in
let default_sandbox_label = "workspace-readonly" in
let requested_capability_label = "exec:outside-sandbox" in
let escalation_reason_label = "process-adapter-request" in
let safe_rationale = "bounded command prefix" in
let prompt_rationale = "host approval required" in
let forbid_rationale = "destructive prefix denied" in
{
  exec_policy | ExecPolicyCatalog = {
    schema = policy_schema,
    sandbox_context = {
      active_sandbox = default_sandbox_label,
      requested_capability = requested_capability_label,
      escalation_reason = escalation_reason_label,
    },
    valid_rules = [
      {
        id = "allow-cargo-readonly",
        decision = "allow",
        rationale = safe_rationale,
        command_prefix = ["cargo"],
        positive_examples = [
          { argv = ["cargo", "metadata"] },
          { argv = ["cargo", "test", "-p", "animus-adapters"] },
        ],
        negative_examples = [
          { argv = ["rm", "-rf", "target"] },
          { argv = ["nix", "build"] },
        ],
      },
      {
        id = "prompt-nix-build",
        decision = "prompt",
        rationale = prompt_rationale,
        command_prefix = ["nix"],
        positive_examples = [
          { argv = ["nix", "build"] },
          { argv = ["nix", "develop"] },
        ],
        negative_examples = [
          { argv = ["cargo", "metadata"] },
          { argv = ["rm", "-rf", "target"] },
        ],
      },
      {
        id = "forbid-destructive-remove",
        decision = "forbid",
        rationale = forbid_rationale,
        command_prefix = ["rm"],
        positive_examples = [
          { argv = ["rm", "-rf", "target"] },
          { argv = ["rm", "-r", "tmp"] },
        ],
        negative_examples = [
          { argv = ["cargo", "metadata"] },
          { argv = ["nix", "build"] },
        ],
      },
    ],
    invalid_rule_fixtures = [
      {
        name = "empty-prefix",
        expected_error = "empty_collection",
        rule = {
          id = "empty-prefix",
          decision = "allow",
          rationale = safe_rationale,
          command_prefix = [],
          positive_examples = [{ argv = ["cargo"] }],
          negative_examples = [{ argv = ["rm"] }],
        },
      },
      {
        name = "unsupported-decision",
        expected_error = "unsupported_decision",
        rule = {
          id = "unsupported-decision",
          decision = "maybe",
          rationale = safe_rationale,
          command_prefix = ["cargo"],
          positive_examples = [{ argv = ["cargo"] }],
          negative_examples = [{ argv = ["rm"] }],
        },
      },
      {
        name = "positive-example-mismatch",
        expected_error = "positive_example_mismatch",
        rule = {
          id = "positive-example-mismatch",
          decision = "allow",
          rationale = safe_rationale,
          command_prefix = ["cargo"],
          positive_examples = [{ argv = ["rm"] }],
          negative_examples = [{ argv = ["rm", "-rf", "target"] }],
        },
      },
      {
        name = "negative-example-matched",
        expected_error = "negative_example_matched",
        rule = {
          id = "negative-example-matched",
          decision = "allow",
          rationale = safe_rationale,
          command_prefix = ["cargo"],
          positive_examples = [{ argv = ["cargo"] }],
          negative_examples = [{ argv = ["cargo", "test"] }],
        },
      },
    ],
  },
}
