# Typed task set of the campaign-manager Harbor-compatible evaluation lane.
#
# Each task is one exploration campaign with hidden candidate outcomes. Executing a candidate
# reveals its signal (0-9); exactly one candidate or offer per task is the target whose execution
# records a finding. Offers are candidates the manager may add; the target is sometimes only
# reachable through an offer.
let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && value != "") in
let Identity =
  std.contract.from_predicate
    (fun value =>
      std.is_string value
      && value != ""
      && std.string.length value <= 256
      && std.string.is_match "^[A-Za-z0-9._:-]+$" value
    )
  in
let Label =
  std.contract.from_predicate
    (fun value => std.is_string value && value != "" && std.string.length value <= 80)
  in
let Positive =
  std.contract.from_predicate
    (fun value => std.is_number value && std.number.is_integer value && value > 0)
  in
let Signal =
  std.contract.from_predicate
    (fun value => std.is_number value && std.number.is_integer value && value >= 0 && value <= 9)
  in
let Candidate = { key | Identity, label | Label, signal | Signal, is_target | Bool } in
let Task = {
  identity | Identity,
  objective | NonEmpty,
  selection_budget | Positive,
  turns | Positive,
  candidates | Array Candidate,
  offers | Array Candidate,
}
in
let TaskSet = {
  schema | NonEmpty,
  profile_id | Identity,
  harbor | { release | NonEmpty, revision | NonEmpty, task_schema | NonEmpty, atif_schema | NonEmpty },
  agent | Identity,
  environment | Identity,
  verifier | Identity,
  rubric | Identity,
  mandate | {
    brief_window | Positive,
    candidate_batch | Positive,
    guidance | Positive,
    revision | Positive,
    retirement | Positive,
    prune | Positive,
  },
  rank_dimensions | Positive,
  tasks | Array Task,
  non_claims | Array NonEmpty,
}
in
let candidate = fun key_value label_value signal_value =>
  { key = key_value, label = label_value, signal = signal_value, is_target = false }
  in
let target = fun key_value label_value signal_value =>
  { key = key_value, label = label_value, signal = signal_value, is_target = true }
  in
{
  campaign_manager_eval_tasks | TaskSet = {
    schema = "animus.campaign-manager-eval-tasks.v1",
    profile_id = "campaign-manager-eval-v1",
    harbor = {
      release = "v0.18.0",
      revision = "527d50deb63a5d279e8c20593c18a2cbc7f61f9e",
      task_schema = "1.3",
      atif_schema = "ATIF-v1.7",
    },
    agent = "agent:animus-campaign-manager",
    environment = "environment:campaign-manager-rollout",
    verifier = "verifier:campaign-history-replay",
    rubric = "rubric:campaign-manager-v1",
    mandate = {
      brief_window = 64,
      candidate_batch = 16,
      guidance = 64,
      revision = 16,
      retirement = 16,
      prune = 64,
    },
    rank_dimensions = 1,
    tasks = [
      {
        identity = "task:json-decoder-hang",
        objective = "Find the input that makes the JSON decoder hang.",
        selection_budget = 4,
        turns = 8,
        candidates = [
          candidate "strings-unicode" "strings/long runs of unicode escapes" 2,
          candidate "numbers-exponent" "numbers/huge exponent literals" 4,
          target "nesting-arrays" "nesting/arrays nested 100000 deep" 9,
          candidate "whitespace-endings" "whitespace/mixed line endings" 1,
        ],
        offers = [
          candidate "nesting-objects" "nesting/objects nested 1000 deep" 6,
          candidate "numbers-negative-zero" "numbers/negative zero and NaN text" 3,
        ],
      },
      {
        identity = "task:config-parser-crash",
        objective = "Find the configuration file that crashes the parser.",
        selection_budget = 6,
        turns = 10,
        candidates = [
          candidate "includes-missing" "includes/missing include target" 3,
          candidate "includes-self" "includes/file that includes itself" 8,
          candidate "keys-duplicate" "keys/duplicate keys in one table" 2,
          candidate "values-long" "values/strings longer than one mebibyte" 1,
        ],
        offers = [
          target "includes-cycle" "includes/three files including each other in a cycle" 9,
          candidate "includes-deep" "includes/includes nested 50 levels deep" 5,
          candidate "keys-unicode" "keys/keys written with unicode escapes" 2,
          candidate "values-binary" "values/binary bytes inside strings" 1,
        ],
      },
      {
        identity = "task:cache-key-collision",
        objective = "Find the request pair that collides in the response cache.",
        selection_budget = 6,
        turns = 10,
        candidates = [
          candidate "headers-vary" "headers/vary on the cookie header only" 2,
          candidate "paths-slash" "paths/trailing slash variants" 7,
          candidate "query-order" "query/reordered query parameters" 4,
          candidate "methods-head" "methods/HEAD versus GET" 1,
          candidate "encoding-gzip" "encoding/gzip versus identity bodies" 3,
        ],
        offers = [
          target "paths-encoded-slash" "paths/percent-encoded slash in a segment" 9,
          candidate "query-duplicate" "query/duplicate query parameters" 5,
          candidate "headers-case" "headers/case-folded header names" 2,
        ],
      },
    ],
    non_claims = [
      "not-advice-quality-beyond-these-tasks",
      "not-benchmark-superiority",
      "not-verifier-soundness-beyond-history-replay",
      "not-harbor-cloud-or-hub-parity",
      "not-production-or-release-readiness",
    ],
  },
}
