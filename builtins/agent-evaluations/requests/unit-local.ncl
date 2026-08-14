# Reviewed local planning input. This file grants no live authority.
let Identity = String in
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
let Request = {
  selected_case_ids | Array String,
  subject_observations | Array SubjectObservation,
  available_services | Array String,
  budget_availability | Array BudgetAvailability,
  live_authority | Array LiveAuthority,
  prior_trials | Array Dyn,
} in
({
  selected_case_ids = ["unit-local"],
  subject_observations = [{
    case_id = "unit-local",
    subject_identity = "b3:1a8e549ec9033bcaef44c752189fd7de71407315a65cdec48468035928cc26a8",
  }],
  available_services = ["workspace", "agent", "verifier", "artifact", "clock", "store"],
  budget_availability = [{
    case_id = "unit-local",
    trials = 2,
    concurrency = 2,
    tokens = 4096,
    duration_ms = 60000,
    deployments = 0,
  }],
  live_authority = [],
  prior_trials = [],
}) | Request
