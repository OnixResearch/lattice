let ProjectionPolicy = import "../../../policy/structured-input-projection.ncl" in
{
  pi = "toon_v3",
  clankers = "json_compact",
  animus = "json_pretty",
} | ProjectionPolicy
