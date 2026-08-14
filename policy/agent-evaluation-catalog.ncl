# Canonical authoring contract for Lattice agent-evaluation catalogs.
let Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == 67
) in
let Subject = {
  repository_identity | Identity,
  workload_revision | Identity,
  fixture_identity | Identity,
  scaffold_identity | Identity,
  initial_state_identity | Identity,
  hidden_material | [| 'none, 'frozen |],
  mutation_scope | Array String,
} in
let Environment = {
  identity | Identity,
  network | [| 'none, 'isolated, 'live |],
  isolated | Bool,
  target_class | String,
} in
let Budget = {
  trials | Number,
  concurrency | Number,
  max_tokens | Number,
  max_duration_ms | Number,
  max_deployments | Number,
} in
let Gate = {
  id | String,
  implementation_identity | Identity,
  input_identity | Identity,
  evidence_schema | String,
  objective | Bool,
} in
let Rubric = {
  identity | Identity,
  judge_profile_identity | Identity,
  advisory | Bool,
} in
let Reference = {
  identity | Identity,
  source_identity | Identity,
  capture_class | String,
  freshness_required | Bool,
  fresh | Bool,
  agent_controlled | Bool,
} in
let ArtifactPolicy = {
  identity | Identity,
  allowed_roles | Array [| 'source_snapshot, 'result, 'bounded_log, 'trajectory, 'screenshot, 'api_sample, 'deployment_observation |],
  allowed_media | Array [| 'text, 'json, 'image, 'binary |],
  max_count | Number,
  max_bytes | Number,
  retention | [| 'ephemeral, 'reviewed_private |],
  private_destination | Bool,
  redaction_required | Bool,
} in
let LivePosture = {
  enabled | Bool,
  operator_actor | optional | String,
  target_namespace | optional | String,
  credential_handles | Array String,
  retention | [| 'ephemeral, 'reviewed_private |],
  cleanup_required | Bool,
  unknown_outcome | [| 'retain_unknown, 'cleanup_required |],
} in
let Case = {
  id | String,
  class | [| 'unit, 'skill, 'hard, 'live |],
  asset_path | String,
  asset_identity | Identity,
  subject | Subject,
  route | [| 'agent, 'workflow |],
  agent_profile_identity | Identity,
  environment | Environment,
  policy_identity | Identity,
  budget | Budget,
  gates | Array Gate,
  rubric | optional | Rubric,
  references | Array Reference,
  artifacts | ArtifactPolicy,
  live | LivePosture,
  non_claims | Array String,
} in
{ schema | String, cases | Array Case }
