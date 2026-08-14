# Built-in guarded Octet one-agent round.
let ContentIdentity = String in
let FileBinding = { path | String, blake3 | ContentIdentity } in
let AssetBinding = { path | String, media_kind | String, blake3 | ContentIdentity, required | Bool } in
let Compatibility = {
  workflow_schema | Number,
  orchestration_schema | optional | Number,
  node_kinds | Array String,
  runtime_kinds | Array String,
  feature_ids | Array String,
} in
let WorkflowTemplateManifest = {
  schema_version | Number,
  id | String,
  display_name | String,
  description | String,
  category | String,
  workflow | FileBinding,
  orchestration | optional | FileBinding,
  assets | Array AssetBinding,
  compatibility | Compatibility,
  operator_notes | Array String,
  non_claims | Array String,
} in
({
  schema_version = 1,
  id = "octet-agent-burndown",
  display_name = "Octet agent burn-down",
  description = "Run one visible no-disabled Octet refactor round with at most one Pi agent.",
  category = "guarded-agents",
  workflow = {
    path = "workflow.ncl",
    blake3 = "1dadbc1aa533861862ce29f1baba5eb309961793895bd927d8ef65bf78000885",
  },
  orchestration = {
    path = "orchestration.ncl",
    blake3 = "4036801e8585377dcbce57fe7eea75ed9e506fc66863c294ab83b1979db7b0d0",
  },
  assets = [{
    path = "prompts/octet-agent-burndown.md",
    media_kind = "markdown-role",
    blake3 = "e1099b26a401827632cb55120035e9f215526287d2035dfc29b27fe919ba2b15",
    required = true,
  }],
  compatibility = {
    workflow_schema = 1,
    orchestration_schema = 1,
    node_kinds = ["shell", "agent", "merge"],
    runtime_kinds = ["shell", "agent", "merge"],
    feature_ids = ["lattice.workflow-template-bundle.v1", "lattice.guarded-agent-round.v1"],
  },
  operator_notes = [
    "Preview the fixed root, probes, validation, Pi target, prompt, effects, and positive budgets before save or start.",
    "Each probe uses an owned backup and restores exact policy bytes before an agent can run or progress can be accepted.",
    "A recovery mismatch blocks for operator inspection. Candidate edits remain in place after a flat or regressing result.",
  ],
  non_claims = [
    "Bundle identities prove exact reviewed local bytes only.",
    "A lower finding count does not prove correctness, effect uniqueness, or release readiness.",
    "Temporary policy replacement has a host-crash window; recovery blocks instead of claiming arbitrary kill safety.",
    "Repository-root selection grants authority; it does not sandbox arbitrary agent behavior.",
  ],
}) | WorkflowTemplateManifest
