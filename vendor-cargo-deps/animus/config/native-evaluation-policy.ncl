# Typed authority-free native evaluation policy.
let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && value != "") in
let Positive = std.contract.from_predicate (fun value => std.is_number value && value > 0) in
let Blake3Identity = std.contract.from_predicate (fun value => std.is_string value && std.string.is_match "^b3:[0-9a-f]{64}$" value) in
let Policy = {
  schema | NonEmpty,
  profile_id | NonEmpty,
  source_cohort | {
    repository | NonEmpty,
    revision | NonEmpty,
    license | NonEmpty,
    identity | Blake3Identity,
  },
  bounds | {
    tasks | Positive,
    runs | Positive,
    reward_dimensions | Positive,
    artifacts | Positive,
  },
  defaults | {
    network | [| 'none |],
    secret_values_allowed | Bool,
    transcript_bodies_allowed | Bool,
    runtime_nickel_allowed | Bool,
  },
  taskset | {
    stable_order_required | Bool,
    lazy_generator_identity_required | Bool,
    lazy_generator_version_required | Bool,
    explicit_seed_required | Bool,
    positive_selection_bound_required | Bool,
    global_shuffle_allowed | Bool,
  },
  recipes | Array [|
    'single_agent,
    'best_of_n,
    'user_simulation,
    'isolated_agentic_judge,
    'shared_advisory_judge,
  |],
  rewards | {
    missing | [| 'error |],
    non_finite | [| 'error |],
    out_of_domain | [| 'error |],
    judge | [| 'advisory |],
  },
  resume | {
    mismatch | [| 'rerun |],
    require_complete | Bool,
    require_terminal | Bool,
    require_safe | Bool,
  },
  evidence | {
    redaction | [| 'metadata_only |],
    raw_content_allowed | Bool,
    completion_claim_allowed | Bool,
    non_claims | Array NonEmpty,
  },
} in
{
  native_evaluation_policy | Policy = {
    schema = "animus.native-evaluation-policy.v1",
    profile_id = "native-evaluation-v1",
    source_cohort = {
      repository = "PrimeIntellect-ai/verifiers",
      revision = "1a997814e85a2e4f409add960aa852737388dbd8",
      license = "MIT",
      identity = "b3:069f20983a0d523d84028b76f85b3dd63a712f952e99cdef628824ef1ffe6bfe",
    },
    bounds = {
      tasks = 4096,
      runs = 1024,
      reward_dimensions = 128,
      artifacts = 256,
    },
    defaults = {
      network = 'none,
      secret_values_allowed = false,
      transcript_bodies_allowed = false,
      runtime_nickel_allowed = false,
    },
    taskset = {
      stable_order_required = true,
      lazy_generator_identity_required = true,
      lazy_generator_version_required = true,
      explicit_seed_required = true,
      positive_selection_bound_required = true,
      global_shuffle_allowed = false,
    },
    recipes = [
      'single_agent,
      'best_of_n,
      'user_simulation,
      'isolated_agentic_judge,
      'shared_advisory_judge,
    ],
    rewards = {
      missing = 'error,
      non_finite = 'error,
      out_of_domain = 'error,
      judge = 'advisory,
    },
    resume = {
      mismatch = 'rerun,
      require_complete = true,
      require_terminal = true,
      require_safe = true,
    },
    evidence = {
      redaction = 'metadata_only,
      raw_content_allowed = false,
      completion_claim_allowed = false,
      non_claims = [
        "not-task-correctness",
        "not-trace-completeness",
        "not-verifier-soundness",
        "not-benchmark-or-training-fitness",
        "not-authoritative-completion",
        "not-production-or-release-readiness",
      ],
    },
  },
}
