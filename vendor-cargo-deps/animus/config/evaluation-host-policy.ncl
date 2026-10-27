# Typed explicit standard-library evaluation host policy.
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
  concurrency | {
    episodes | Positive,
    agents_per_episode | Positive,
  },
  gateway | {
    bind_modes | Array [| 'loopback, 'caller_admitted |],
    max_body_bytes | Positive,
    max_stream_events | Positive,
    separate_capabilities_required | Bool,
    accepted_turn_required | Bool,
    terminal_commit_required | Bool,
    exact_non_streaming_coalescing_only | Bool,
    response_rewriting_allowed | Bool,
    wildcard_network_allowed | Bool,
    raw_provider_bodies_allowed | Bool,
  },
  verifier | {
    isolated_required | Bool,
    shared_is_advisory | Bool,
    hidden_material_in_solver_allowed | Bool,
  },
  artifacts | {
    max_entries | Positive,
    max_bytes | Positive,
    allowlist_required | Bool,
    no_follow_required | Bool,
    hidden_tests_allowed | Bool,
  },
  persistence | {
    max_record_bytes | Positive,
    durability | [| 'process_observed, 'file_synced, 'file_and_directory_synced |],
    complete_bytes_required | Bool,
    atomic_replace_required | Bool,
    strict_reopen_required | Bool,
    malformed_record_can_skip | Bool,
  },
  cleanup | {
    max_obligations | Positive,
    hide_post_terminal_failure | Bool,
    late_writes_allowed | Bool,
  },
  evidence | {
    metadata_only | Bool,
    raw_content_allowed | Bool,
    physical_paths_allowed | Bool,
    completion_claim_allowed | Bool,
    non_claims | Array NonEmpty,
  },
} in
{
  evaluation_host_policy | Policy = {
    schema = "animus.native-evaluation-host.v1",
    profile_id = "native-evaluation-host-v1",
    source_cohort = {
      repository = "PrimeIntellect-ai/verifiers",
      revision = "1a997814e85a2e4f409add960aa852737388dbd8",
      license = "MIT",
      identity = "b3:c6336b188596db9b18cb2f0a721c9f5bc551e619342e89807e20ec45f3bb676f",
    },
    concurrency = {
      episodes = 32,
      agents_per_episode = 16,
    },
    gateway = {
      bind_modes = ['loopback, 'caller_admitted],
      max_body_bytes = 16777216,
      max_stream_events = 65536,
      separate_capabilities_required = true,
      accepted_turn_required = true,
      terminal_commit_required = true,
      exact_non_streaming_coalescing_only = true,
      response_rewriting_allowed = false,
      wildcard_network_allowed = false,
      raw_provider_bodies_allowed = false,
    },
    verifier = {
      isolated_required = true,
      shared_is_advisory = true,
      hidden_material_in_solver_allowed = false,
    },
    artifacts = {
      max_entries = 256,
      max_bytes = 67108864,
      allowlist_required = true,
      no_follow_required = true,
      hidden_tests_allowed = false,
    },
    persistence = {
      max_record_bytes = 67108864,
      durability = 'file_synced,
      complete_bytes_required = true,
      atomic_replace_required = true,
      strict_reopen_required = true,
      malformed_record_can_skip = false,
    },
    cleanup = {
      max_obligations = 256,
      hide_post_terminal_failure = false,
      late_writes_allowed = false,
    },
    evidence = {
      metadata_only = true,
      raw_content_allowed = false,
      physical_paths_allowed = false,
      completion_claim_allowed = false,
      non_claims = [
        "not-host-confinement-or-network-enforcement",
        "not-provider-or-tool-correctness",
        "not-persistence-durability-or-exactly-once-effects",
        "not-verifier-soundness-or-artifact-authenticity",
        "not-benchmark-or-training-fitness",
        "not-authoritative-completion",
        "not-production-or-release-readiness",
      ],
    },
  },
}
