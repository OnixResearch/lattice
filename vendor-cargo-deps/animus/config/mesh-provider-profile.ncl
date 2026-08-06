# Typed private Mesh LLM gateway profile. Runtime transport remains injected.
let Positive = std.contract.from_predicate (fun value => std.is_number value && value > 0) in
let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && value != "") in
let Profile = {
  schema | NonEmpty,
  profile_id | NonEmpty,
  provider_label | NonEmpty,
  api_style | [| 'responses |],
  gateway_posture | [| 'private_only |],
  exact_model_required | Bool,
  denied_virtual_models | Array NonEmpty,
  provider_fallback | [| 'denied |],
  unknown_capability | [| 'deny_required |],
  catalog | { max_models | Positive, max_model_id_bytes | Positive },
  response | { max_bytes | Positive },
  redaction | [| 'metadata_only |],
  physical_route | [| 'unknown |],
  non_claims | Array NonEmpty,
} in
let maximum_models = 128 in
let maximum_model_id_bytes = 128 in
let maximum_response_bytes = 8388608 in
{
  mesh_provider_profile | Profile = {
    schema = "animus.mesh-provider-profile.v1",
    profile_id = "private-responses-exact-model-v1",
    provider_label = "mesh-private",
    api_style = 'responses,
    gateway_posture = 'private_only,
    exact_model_required = true,
    denied_virtual_models = ["auto", "mesh"],
    provider_fallback = 'denied,
    unknown_capability = 'deny_required,
    catalog = { max_models = maximum_models, max_model_id_bytes = maximum_model_id_bytes },
    response = { max_bytes = maximum_response_bytes },
    redaction = 'metadata_only,
    physical_route = 'unknown,
    non_claims = [
      "not-remote-runtime-integrity",
      "not-peer-honesty",
      "not-physical-route-truth",
      "not-model-correctness",
      "not-split-parity",
      "not-production-readiness",
    ],
  },
}
