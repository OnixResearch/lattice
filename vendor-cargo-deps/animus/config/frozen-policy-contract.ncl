# Authoring contract of `animus.frozen-policy.v1` artifacts.
#
# `animus-frozen-policy` re-checks every rule at admission and is authoritative; this contract
# catches authoring mistakes before export. Keys mirror the Rust document; Nickel exports them
# sorted, and admission re-encodes them canonically, so neither order nor spacing is identity.
let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && value != "") in
let Label =
  std.contract.from_predicate
    (fun value => std.is_string value && std.string.is_match "^[A-Za-z0-9._:-]{1,64}$" value)
  in
let Integer =
  std.contract.from_predicate
    (fun value => std.is_number value && std.number.is_integer value)
  in
let Parameter =
  std.contract.from_predicate
    (fun value =>
      std.is_number value
      && std.number.is_integer value
      && value >= -1000000
      && value <= 1000000
    )
  in
let Index =
  std.contract.from_predicate
    (fun value => std.is_number value && std.number.is_integer value && value > 0 && value < 255)
  in
let PerMille =
  std.contract.from_predicate
    (fun value => std.is_number value && std.number.is_integer value && value >= 0 && value <= 1000)
  in
let Budget =
  std.contract.from_predicate
    (fun value => std.is_number value && std.number.is_integer value && value > 0 && value <= 1000000)
  in
let Node = {
  leaf | { scores | Array Parameter } | optional,
  split | { above | Index, below | Index, feature | Integer, threshold | PerMille } | optional,
}
in
let Layer = {
  activation | [| 'relu, 'identity |],
  biases | Array Parameter,
  weights | Array (Array Parameter),
}
in
{
  Artifact = {
    schema | std.contract.Equal "animus.frozen-policy.v1",
    runtime | { crate | std.contract.Equal "animus-frozen-policy", artifact_api | std.contract.Equal 1 },
    feature_schema | {
      id | Label,
      normalization | std.contract.Equal "min-max-per-mille-v1",
      features | Array { name | Label, min | Integer, max | Integer },
    },
    action_schema | { id | Label, actions | Array Label },
    model | {
      kind | [| '"decision-tree-v1", '"mlp-per-mille-v1" |],
      nodes | Array Node | optional,
      layers | Array Layer | optional,
    },
    bounds | {
      score_min | Parameter,
      score_max | Parameter,
      abstain_below | Parameter | optional,
      operation_budget | Budget,
    },
    provenance | {
      kind | [| 'fixture, 'trained, 'distilled |],
      producer | NonEmpty,
      evaluation | NonEmpty,
      non_claims | Array NonEmpty,
    },
  },
}
