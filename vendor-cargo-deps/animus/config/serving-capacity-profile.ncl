# Typed capacity-aware serving profile. Snapshots, the serving owner, and the
# transport remain host-injected; this profile only selects reviewed rules.
# Rust (`animus_std::serving::parse_profile`) revalidates every exported value.
let contracts = import "../policy/contracts/primitives.ncl" in

let schema_identity = "animus.serving-capacity-profile.v1" in
let maximum_snapshot_age_limit_ticks = 3600 in
let reviewed_bounds = {
  max_services = 64,
  max_capacity_observations = 16,
  max_route_candidates = 32,
  max_rollout_operations = 8,
  max_token_bytes = 256,
  max_snapshot_bytes = 262144,
} in
let reviewed_evidence_labels = ["fixture", "live-endpoint"] in
let reviewed_non_claims = [
  "not-gpu-scheduling",
  "not-production-availability",
  "not-physical-placement-truth",
  "not-model-correctness",
  "not-capacity-reservation",
  "not-provider-honesty",
  "not-cost-or-pricing",
] in

let ProfileShape = {
  schema | contracts.exact "SERVING_PROFILE_SCHEMA" schema_identity,
  profile_id | contracts.safe_identifier "SERVING_PROFILE_ID",
  identity_rule
    | contracts.enum "SERVING_PROFILE_IDENTITY_RULE" ["claimed_accepted", "observed_required"],
  capacity_rule
    | contracts.enum "SERVING_PROFILE_CAPACITY_RULE" ["known_required", "owner_admission_decides"],
  replica_rule | contracts.enum "SERVING_PROFILE_REPLICA_RULE" ["same_model_allowed", "pinned"],
  max_snapshot_age_ticks
    | contracts.integer_between "SERVING_PROFILE_AGE_BOUND" 1 maximum_snapshot_age_limit_ticks,
  owner_admission | contracts.exact "SERVING_PROFILE_OWNER_ADMISSION" "required",
  redaction | contracts.exact "SERVING_PROFILE_REDACTION" "metadata_only",
  evidence_labels | contracts.exact "SERVING_PROFILE_EVIDENCE_LABELS" reviewed_evidence_labels,
  bounds | contracts.exact "SERVING_PROFILE_BOUNDS" reviewed_bounds,
  non_claims | contracts.exact "SERVING_PROFILE_NON_CLAIMS" reviewed_non_claims,
} in

{
  serving_capacity_profile_contract = ProfileShape,
  serving_capacity_profile | ProfileShape = {
    schema = schema_identity,
    profile_id = "capacity-aware-observed-v1",
    identity_rule = "observed_required",
    capacity_rule = "known_required",
    replica_rule = "same_model_allowed",
    max_snapshot_age_ticks = 30,
    owner_admission = "required",
    redaction = "metadata_only",
    evidence_labels = reviewed_evidence_labels,
    bounds = reviewed_bounds,
    non_claims = reviewed_non_claims,
  },
}
