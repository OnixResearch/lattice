# Hand-authored fixture artifact for the campaign manager's frozen local scorer route.
#
# It qualifies the route and artifact boundary (Animus #15, first pilot); it was not trained.
# Its weights encode an explore-then-exploit heuristic over the declared per-mille brief
# features: unexecuted candidates outrank executed ones, members of the family holding the best
# observed signal rank highest, and executed candidates follow their own signal.
# score = bias + (sum of weight * feature) / 1000, in checked integer arithmetic.
let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && value != "") in
let Parameter =
  std.contract.from_predicate
    (fun value =>
      std.is_number value
      && std.number.is_integer value
      && value >= -1000000
      && value <= 1000000
    )
  in
let Positive =
  std.contract.from_predicate
    (fun value => std.is_number value && std.number.is_integer value && value > 0)
  in
let Artifact = {
  schema | NonEmpty,
  policy | NonEmpty,
  feature_schema | NonEmpty,
  action_schema | NonEmpty,
  features | Array NonEmpty,
  weights | Array Parameter,
  bias | Parameter,
  rank_bounds | { min | Parameter, max | Parameter },
  runtime | { crate | NonEmpty, artifact_api | Positive, operation_budget | Positive },
  provenance | {
    kind | NonEmpty,
    producer | NonEmpty,
    evaluation | NonEmpty,
    non_claims | Array NonEmpty,
  },
}
in
{
  campaign_manager_local_scorer_fixture | Artifact = {
    schema = "animus.campaign-manager-local-scorer.v1",
    policy = "linear-per-mille-v1",
    feature_schema = "animus.campaign-manager-brief-features.v1",
    action_schema = "animus.campaign-manager-rank-action.v1",
    features = [
      "executed",
      "signal",
      "family_best_signal",
      "family_unexplored",
      "brief_unexplored",
      "family_leads",
    ],
    weights = [-10, 9, 5, 0, 0, 3],
    bias = 10,
    rank_bounds = { min = -100, max = 100 },
    # 32 listed candidates times 6 features: every admitted brief fits.
    runtime = { crate = "animus-campaign-manager-eval", artifact_api = 1, operation_budget = 192 },
    provenance = {
      kind = "fixture",
      producer = "animus:add-frozen-local-scorer",
      evaluation = "none: hand-authored to qualify the route boundary",
      non_claims = [
        "not trained or distilled",
        "encodes a hand-written heuristic, so its results do not measure learning",
        "results on the synthetic evaluator do not establish consumer improvement",
      ],
    },
  },
}
