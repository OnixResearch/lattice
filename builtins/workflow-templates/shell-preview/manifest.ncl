# Built-in Lattice workflow-template bundle.
let ContentIdentity = String in
let FileBinding = {
  path | String,
  blake3 | ContentIdentity,
} in
let AssetBinding = {
  path | String,
  media_kind | String,
  blake3 | ContentIdentity,
  required | Bool,
} in
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
let bundle_schema_version = 1 in
let workflow_schema_version = 1 in
(
  {
    schema_version = bundle_schema_version,
    id = "shell-preview",
    display_name = "Shell preview",
    description = "Preview one fixed shell node as an unsaved workflow draft.",
    category = "examples",
    workflow = {
      path = "workflow.ncl",
      blake3 = "ac10d0357c3fe251f711f81de82735f7eb4b5bd0c9f8a83059137462e260e6c1",
    },
    assets = [],
    compatibility = {
      workflow_schema = workflow_schema_version,
      node_kinds = ["shell"],
      runtime_kinds = ["shell"],
      feature_ids = ["lattice.workflow-template-bundle.v1"],
    },
    operator_notes = [
      "Selection and preview do not save or execute this workflow.",
      "Inspect the command before explicit save or run.",
    ],
    non_claims = [
      "Bundle identity proves exact reviewed local bytes only.",
      "Preview does not prove command safety, result correctness, or release readiness.",
    ],
  }
) | WorkflowTemplateManifest
