# Typed central policy source for the optional Bubblewrap agent guardrail.
# r[impl agent_harness_runtime.bubblewrap_agent_guardrail.profile]
let contracts = import "../policy/contracts/primitives.ncl" in
let maximum_bound = 1000000 in
let ProfileRoot = {
  label | String,
  target | String,
  mode | contracts.enum "AGENT_GUARDRAIL_ROOT_MODE" ["read-only", "writable"],
  source_kind | contracts.enum "AGENT_GUARDRAIL_SOURCE_KIND" ["directory", "file"],
  required | Bool,
} in
let Profile = {
  schema | contracts.exact "AGENT_GUARDRAIL_SCHEMA" "animus.agent-guardrail.profile.v1",
  label | String,
  backend | contracts.exact "AGENT_GUARDRAIL_BACKEND" "bubblewrap-agent-guardrail",
  cohort | contracts.exact "AGENT_GUARDRAIL_COHORT" "bubblewrap-0.11.2-args-v1",
  trust_source | contracts.exact "AGENT_GUARDRAIL_TRUST" "central-reviewed",
  roots | Array ProfileRoot,
  network | contracts.enum "AGENT_GUARDRAIL_NETWORK" ["deny", "inherit"],
  devices | Bool,
  host_sockets | Bool,
  environment_keys | Array String,
  bounds = {
    max_roots | contracts.positive_integer "AGENT_GUARDRAIL_MAX_ROOTS" maximum_bound,
    max_environment | contracts.positive_integer "AGENT_GUARDRAIL_MAX_ENVIRONMENT" maximum_bound,
    max_options | contracts.positive_integer "AGENT_GUARDRAIL_MAX_OPTIONS" maximum_bound,
  },
  escape_hatches = {
    raw_arguments | Bool,
    command_prefix | Bool,
    passthrough | Bool,
    command_environment_source | Bool,
    implicit_shell | Bool,
    unguarded_fallback | Bool,
    privileged_sidecar | Bool,
    project_local_policy | Bool,
  },
  identity = {
    algorithm | contracts.exact "AGENT_GUARDRAIL_HASH" "blake3",
    profile_domain | contracts.exact "AGENT_GUARDRAIL_PROFILE_DOMAIN" "animus.agent-guardrail.profile.v1",
    plan_domain | contracts.exact "AGENT_GUARDRAIL_PLAN_DOMAIN" "animus.agent-guardrail.plan.v1",
    executable_domain | contracts.exact "AGENT_GUARDRAIL_EXECUTABLE_DOMAIN" "animus.agent-guardrail.executable.v1",
    receipt_domain | contracts.exact "AGENT_GUARDRAIL_RECEIPT_DOMAIN" "animus.agent-guardrail.receipt.v1",
  },
  redaction = {
    marker | contracts.exact "AGENT_GUARDRAIL_REDACTION" "raw-prompt-credential-environment-host-path-mount-command-output-debug-and-unbounded-events-omitted",
    raw_payloads | Bool,
  },
  non_claims | Array String,
} in
let validate = fun value =>
  if value.devices || value.host_sockets then
    'Error { message = "AGENT_GUARDRAIL_BROAD_EXPOSURE" }
  else if value.escape_hatches.raw_arguments
    || value.escape_hatches.command_prefix
    || value.escape_hatches.passthrough
    || value.escape_hatches.command_environment_source
    || value.escape_hatches.implicit_shell
    || value.escape_hatches.unguarded_fallback
    || value.escape_hatches.privileged_sidecar
    || value.escape_hatches.project_local_policy then
    'Error { message = "AGENT_GUARDRAIL_ESCAPE_HATCH" }
  else if std.array.length value.roots > value.bounds.max_roots
    || std.array.length value.environment_keys > value.bounds.max_environment then
    'Error { message = "AGENT_GUARDRAIL_BOUND" }
  else if std.array.length value.non_claims < 8 then
    'Error { message = "AGENT_GUARDRAIL_NON_CLAIMS" }
  else
    'Ok
in
let GuardrailProfile = contracts.with_validator Profile validate in
{
  agent_guardrail_profile_contract = GuardrailProfile,
  agent_guardrail_profile | GuardrailProfile = {
    schema = "animus.agent-guardrail.profile.v1",
    label = "interactive-agent-default",
    backend = "bubblewrap-agent-guardrail",
    cohort = "bubblewrap-0.11.2-args-v1",
    trust_source = "central-reviewed",
    roots = [
      {
        label = "tool-store",
        target = "/nix/store",
        mode = "read-only",
        source_kind = "directory",
        required = true,
      },
      {
        label = "worktree",
        target = "/workspace",
        mode = "writable",
        source_kind = "directory",
        required = true,
      },
      {
        label = "guardrail-fixture",
        target = "/guardrail-fixture",
        mode = "read-only",
        source_kind = "file",
        required = false,
      },
    ],
    network = "deny",
    devices = false,
    host_sockets = false,
    environment_keys = ["TERM"],
    bounds = { max_roots = 32, max_environment = 16, max_options = 256 },
    escape_hatches = {
      raw_arguments = false,
      command_prefix = false,
      passthrough = false,
      command_environment_source = false,
      implicit_shell = false,
      unguarded_fallback = false,
      privileged_sidecar = false,
      project_local_policy = false,
    },
    identity = {
      algorithm = "blake3",
      profile_domain = "animus.agent-guardrail.profile.v1",
      plan_domain = "animus.agent-guardrail.plan.v1",
      executable_domain = "animus.agent-guardrail.executable.v1",
      receipt_domain = "animus.agent-guardrail.receipt.v1",
    },
    redaction = {
      marker = "raw-prompt-credential-environment-host-path-mount-command-output-debug-and-unbounded-events-omitted",
      raw_payloads = false,
    },
    non_claims = [
      "not-hostile-code-containment",
      "not-kernel-correctness",
      "not-same-uid-secrecy",
      "not-filtered-network-egress",
      "not-authorization",
      "not-build-hermeticity",
      "not-toctou-safe-path-resolution",
      "not-production-or-release-readiness",
    ],
  },
}
