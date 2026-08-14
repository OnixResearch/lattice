# Canonical authoring contract for Lattice agent-evaluation planning inputs.
let Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == 67
) in
let SubjectObservation = {
  case_id | String,
  subject_identity | Identity,
} in
let BudgetAvailability = {
  case_id | String,
  trials | Number,
  concurrency | Number,
  tokens | Number,
  duration_ms | Number,
  deployments | Number,
} in
let LiveAuthority = {
  case_id | String,
  actor | String,
  target_namespace | String,
  credential_handles | Array String,
  max_deployments | Number,
} in
let PriorTrial = {
  trial_identity | Identity,
  case_revision_identity | Identity,
  subject_identity | Identity,
  agent_profile_identity | Identity,
  environment_identity | Identity,
  policy_identity | Identity,
  gate_set_identity | Identity,
  rubric_identity | optional | Identity,
  reference_set_identity | Identity,
  artifact_policy_identity | Identity,
  status | [| 'planned, 'started, 'passed, 'failed, 'cancelled, 'partial, 'cleanup_required, 'unknown |],
  receipt_identity | optional | Identity,
  expected_receipt_identity | optional | Identity,
} in
{
  selected_case_ids | Array String,
  subject_observations | Array SubjectObservation,
  available_services | Array [| 'workspace, 'agent, 'workflow, 'verifier, 'judge, 'deployment, 'artifact, 'cleanup, 'clock, 'store |],
  budget_availability | Array BudgetAvailability,
  live_authority | Array LiveAuthority,
  prior_trials | Array PriorTrial,
}
