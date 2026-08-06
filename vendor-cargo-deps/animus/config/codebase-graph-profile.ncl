# Typed Rust-first codebase-graph profile. Host paths and parser effects remain external.
let Positive = std.contract.from_predicate (fun value => std.is_number value && value > 0) in
let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && value != "") in
let Profile = {
  schema | NonEmpty,
  profile_id | NonEmpty,
  language | NonEmpty,
  parser_cohort | NonEmpty,
  query_cohort | NonEmpty,
  capability_root_label | NonEmpty,
  include_extensions | Array NonEmpty,
  exclude_components | Array NonEmpty,
  budgets | {
    max_files | Positive,
    max_file_bytes | Positive,
    max_results | Positive,
  },
  stale_event_policy | [| 'block_until_rescan |],
  cache_posture | [| 'disabled, 'capability_scoped_disposable |],
  binary_policy | [| 'reject |],
  special_file_policy | [| 'reject |],
  redaction | [| 'metadata_only |],
  non_claims | Array NonEmpty,
} in
let maximum_files = 4096 in
let maximum_file_bytes = 1048576 in
let maximum_results = 128 in
{
  codebase_graph_profile | Profile = {
    schema = "animus.codebase-graph-profile.v1",
    profile_id = "rust-first",
    language = "rust",
    parser_cohort = "tree-sitter-rust-0.24",
    query_cohort = "rust-query-v1",
    capability_root_label = "worktree",
    include_extensions = ["rs"],
    exclude_components = [".git", ".jj", "target"],
    budgets = {
      max_files = maximum_files,
      max_file_bytes = maximum_file_bytes,
      max_results = maximum_results,
    },
    stale_event_policy = 'block_until_rescan,
    cache_posture = 'disabled,
    binary_policy = 'reject,
    special_file_policy = 'reject,
    redaction = 'metadata_only,
    non_claims = [
      "not-semantic-analysis",
      "not-type-correctness",
      "not-complete-reference-resolution",
      "not-mutation-authority",
      "not-production-readiness",
      "not-release-eligibility",
    ],
  },
}
