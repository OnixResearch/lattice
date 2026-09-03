let Positive = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let RuntimeProfile = {
  schema | String,
  model | String,
  endpoint_class | [| 'controlled_local_openai_compatible |],
  endpoint | String,
  temperature_milliunits | Number,
  max_tokens | Positive,
  timeout_secs | Positive,
  output_mode | [| 'json |],
  system_prompt_path | String,
  corpus_path | String,
  comparator_path | String,
  compared_profiles | Array [| 'json_compact, 'toon_v3 |],
  retains_raw_prompts | Bool,
  claims_provider_billing_truth | Bool,
  claims_production_readiness | Bool,
} in
RuntimeProfile
