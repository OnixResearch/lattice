# Built-in guarded Cairn one-agent round.
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
  id = "cairn-agent-drain",
  display_name = "Cairn agent drain",
  description = "Run one visible, validated Cairn drain round with at most one Pi agent.",
  category = "guarded-agents",
  workflow = {
    path = "workflow.ncl",
    blake3 = "3dc4092a5cea6db8dd2368b86bf55711e9ac9f5a8d1226d1b7723b3ceec8fcff",
  },
  orchestration = {
    path = "orchestration.ncl",
    blake3 = "27e5e3ba57a7d33eb90b906c76532a87797d15df6efb0ff1cb2603a0f2937e1c",
  },
  assets = [{
    path = "prompts/cairn-agent-drain.md",
    media_kind = "markdown-role",
    blake3 = "920a8c43388f3265d70dd877736dfc4ce630ac0031fa45db2a9fb7fbde3f54e4",
    required = true,
  }],
  compatibility = {
    workflow_schema = 1,
    orchestration_schema = 1,
    node_kinds = ["shell", "condition", "agent", "merge"],
    runtime_kinds = ["shell", "condition", "agent", "merge"],
    feature_ids = ["lattice.workflow-template-bundle.v1", "lattice.guarded-agent-round.v1"],
  },
  operator_notes = [
    "Preview the fixed root, commands, Pi target, prompt, effects, and positive budgets before save or start.",
    "An empty active set makes no agent call. A successor requires strict count reduction and separate operator approval.",
    "The agent can edit the selected repository and may commit only when repository policy requires it. The bundle never pushes.",
  ],
  non_claims = [
    "Bundle identities prove exact reviewed local bytes only.",
    "Active-count reduction and archive state do not prove correctness, effect uniqueness, or release readiness.",
    "Repository-root selection grants authority; it does not sandbox arbitrary agent behavior.",
  ],
}) | WorkflowTemplateManifest
