# Hand-authored example artifacts of `animus.frozen-policy.v1`, one per policy kind.
#
# They exercise the frozen-policy leaf's admission, evaluation, selection, and receipts on a
# product-neutral triage task: given a work item's attempt count, observed failure rate, and
# queue depth, prefer `proceed`, `retry`, or `escalate`. They were not trained, and the task is
# illustrative: a consumer declares its own feature and action schemas.
let contract = import "frozen-policy-contract.ncl" in
let shared = {
  schema = "animus.frozen-policy.v1",
  runtime = { crate = "animus-frozen-policy", artifact_api = 1 },
  feature_schema = {
    id = "animus.frozen-policy-example-features.v1",
    normalization = "min-max-per-mille-v1",
    features = [
      { name = "attempts", min = 0, max = 10 },
      { name = "failure_rate_per_mille", min = 0, max = 1000 },
      { name = "queue_depth", min = 0, max = 500 },
    ],
  },
  action_schema = {
    id = "animus.frozen-policy-example-actions.v1",
    actions = ["proceed", "retry", "escalate"],
  },
  provenance = {
    kind = 'fixture,
    producer = "animus:add-frozen-policy-leaf",
    evaluation = "none: hand-authored to qualify the leaf's admission and decision boundary",
    non_claims = [
      "not trained or distilled",
      "the triage task is illustrative and measures no consumer outcome",
    ],
  },
}
in
{
  # attempts <= 3 (300 per mille): proceed while failures stay at or below half, else retry;
  # more attempts escalate. Below a best eligible score of 50 the policy abstains.
  frozen_policy_example_tree | contract.Artifact = shared & {
    model = {
      kind = '"decision-tree-v1",
      nodes = [
        { split = { feature = 0, threshold = 300, below = 1, above = 2 } },
        { split = { feature = 1, threshold = 500, below = 3, above = 4 } },
        { leaf = { scores = [0, 10, 90] } },
        { leaf = { scores = [100, 20, 0] } },
        { leaf = { scores = [10, 80, 30] } },
      ],
    },
    bounds = { score_min = -1000, score_max = 1000, abstain_below = 50, operation_budget = 16 },
  },
  # One hidden ReLU layer of four units over the three normalized features, then one identity
  # output per action. Weights are per mille: output = bias + sum(weight * input) / 1000.
  frozen_policy_example_mlp | contract.Artifact = shared & {
    model = {
      kind = '"mlp-per-mille-v1",
      layers = [
        {
          activation = 'relu,
          biases = [0, 0, 1000, 0],
          weights = [
            [1000, 0, 0],
            [0, 1000, 0],
            [-1000, 0, 0],
            [0, 0, 1000],
          ],
        },
        {
          activation = 'identity,
          biases = [0, 0, -300],
          weights = [
            [0, -500, 500, 0],
            [-200, 600, 0, 0],
            [800, 0, 0, 200],
          ],
        },
      ],
    },
    bounds = { score_min = -1000, score_max = 1000, operation_budget = 24 },
  },
}
