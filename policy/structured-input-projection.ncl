# Closed per-target policy for model-visible structured input.
let Projection = std.contract.from_predicate (fun value =>
  std.is_string value
  && (value == "json_pretty" || value == "json_compact" || value == "toon_v3")
) in
{
  pi | optional | Projection,
  clankers | optional | Projection,
  animus | optional | Projection,
}
