# Hand-authored decision-tree fixture for the campaign manager's frozen-policy route.
#
# It qualifies the tree route through the shared `animus-frozen-policy` leaf (Animus #15); it was
# not trained, and it was authored before the comparison ran without reading the evaluation
# tasks. Each listed candidate is classified from its six declared per-mille brief features:
# an executed candidate follows its observed signal; an unexecuted member of the family holding
# the brief's best signal is exploited once that signal exceeds 600 per mille (6 of 9);
# every other unexecuted candidate is explored. The route maps classes to ranks.
let contract = import "frozen-policy-contract.ncl" in
{
  campaign_manager_frozen_policy_fixture | contract.Artifact = {
    schema = "animus.frozen-policy.v1",
    runtime = { crate = "animus-frozen-policy", artifact_api = 1 },
    feature_schema = {
      id = "animus.campaign-manager-brief-features.v1",
      normalization = "min-max-per-mille-v1",
      features = [
        { name = "executed", min = 0, max = 1000 },
        { name = "signal", min = 0, max = 1000 },
        { name = "family_best_signal", min = 0, max = 1000 },
        { name = "family_unexplored", min = 0, max = 1000 },
        { name = "brief_unexplored", min = 0, max = 1000 },
        { name = "family_leads", min = 0, max = 1000 },
      ],
    },
    action_schema = {
      id = "animus.campaign-manager-candidate-class.v1",
      actions = ["follow-signal", "explore", "exploit"],
    },
    model = {
      kind = '"decision-tree-v1",
      nodes = [
        { split = { feature = 0, threshold = 500, below = 1, above = 2 } },
        { split = { feature = 5, threshold = 500, below = 3, above = 4 } },
        { leaf = { scores = [100, 0, 0] } },
        { leaf = { scores = [0, 100, 0] } },
        { split = { feature = 2, threshold = 600, below = 5, above = 6 } },
        { leaf = { scores = [0, 100, 0] } },
        { leaf = { scores = [0, 0, 100] } },
      ],
    },
    # Three splits at most per candidate.
    bounds = { score_min = -1000, score_max = 1000, operation_budget = 8 },
    provenance = {
      kind = 'fixture,
      producer = "animus:add-frozen-policy-leaf",
      evaluation = "none: hand-authored to qualify the tree route boundary",
      non_claims = [
        "not trained or distilled",
        "encodes a hand-written heuristic, so its results do not measure learning",
        "results on the synthetic evaluator do not establish consumer improvement",
      ],
    },
  },
}
