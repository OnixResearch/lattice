# Built-in agent-evaluation catalog review bundle.
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
  id = "agent-evaluation-catalog-review",
  display_name = "Agent evaluation catalog review",
  description = "Validate and display the built-in identity-bound evaluation catalog without executing trials.",
  category = "agent-evaluation",
  workflow = {
    path = "workflow.ncl",
    blake3 = "073c8849e64b76e658bb41481a5cefe7824bfe4c258a737def47f43358c84ab2",
  },
  assets = [],
  compatibility = {
    workflow_schema = 1,
    node_kinds = ["shell"],
    runtime_kinds = ["shell"],
    feature_ids = ["lattice.workflow-template-bundle.v1", "lattice.agent-evaluation-catalog.v1"],
  },
  operator_notes = [
    "This bundle validates catalog identity and case metadata only. It does not start an agent, workflow, deployment, or live trial.",
    "Local and live trials require the product runner with injected services and durable start records.",
    "Live authority, credential references, target namespace, retention, and cleanup remain explicit operator inputs.",
  ],
  non_claims = [
    "Catalog and content identities prove exact bytes only.",
    "Catalog validation does not prove agent, verifier, judge, deployment, or benchmark correctness.",
    "A successful catalog review does not grant live authority or release eligibility.",
  ],
}) | WorkflowTemplateManifest
