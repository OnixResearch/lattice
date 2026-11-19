# Predeclared comparison protocol of the campaign manager's frozen local scorer route.
#
# Arms run on every task of the embedded evaluation task set with the same turns and selection
# budgets: the grant-matched `explore-only` baseline, one random-ranking route per seed, the local
# scorer over the selected artifact, and `signal-greedy` as a reference whose broader grant (it
# adds offers) makes it non-comparable. The fixture artifact used no task data, so no task is
# in-sample. The adoption rule decides only this synthetic lane.
let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && value != "") in
let Digest =
  std.contract.from_predicate
    (fun value => std.is_string value && std.string.is_match "^b3:[0-9a-f]{64}$" value)
  in
let Identity =
  std.contract.from_predicate
    (fun value =>
      std.is_string value
      && value != ""
      && std.string.length value <= 256
      && std.string.is_match "^[A-Za-z0-9._:-]+$" value
    )
  in
let PerMille =
  std.contract.from_predicate
    (fun value => std.is_number value && std.number.is_integer value && value >= 0 && value <= 1000)
  in
let Seed =
  std.contract.from_predicate
    (fun value => std.is_number value && std.number.is_integer value && value > 0)
  in
let Protocol = {
  schema | NonEmpty,
  artifact_identity | Digest,
  baseline | Identity,
  references | Array Identity,
  random_seeds | Array Seed,
  primary_outcome | NonEmpty,
  adoption | {
    min_gain_over_baseline_per_mille | PerMille,
    requires_random_parity | Bool,
    max_fallback_per_mille | PerMille,
  },
  non_claims | Array NonEmpty,
}
in
{
  campaign_manager_local_scorer_eval | Protocol = {
    schema = "animus.campaign-manager-local-scorer-eval.v1",
    artifact_identity = "b3:4a2c69d0ee7b5f67c14e553892a2038ea6cf82d39fe29ebbd75602b5abc1d595",
    baseline = "model:scripted:explore-only",
    references = ["model:scripted:signal-greedy"],
    random_seeds = [1, 2, 3, 4, 5, 6, 7, 8],
    primary_outcome = "found",
    adoption = {
      min_gain_over_baseline_per_mille = 100,
      requires_random_parity = true,
      max_fallback_per_mille = 0,
    },
    non_claims = [
      "the decision covers only the synthetic evaluator's three tasks",
      "a fixture artifact's result does not measure learning or a trained model",
      "scores and ranks are neither task success nor authorization",
      "production rollout stays conditional on a trained artifact and the consumer pilot",
    ],
  },
}
