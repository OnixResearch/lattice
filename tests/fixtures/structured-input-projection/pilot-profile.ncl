let RuntimeProfile = import "../../../policy/structured-input-projection-pilot.ncl" in
let model_token_limit = 8192 in
let request_timeout_secs = 600 in
{
  schema = "lattice.structured-input-projection-pilot-profile.v1",
  model = "VibeThinker-3B",
  endpoint_class = 'controlled_local_openai_compatible,
  endpoint = "http://127.0.0.1:13305/v1/chat/completions",
  temperature_milliunits = 0,
  max_tokens = model_token_limit,
  timeout_secs = request_timeout_secs,
  output_mode = 'json,
  system_prompt_path = "tests/fixtures/structured-input-projection/system-prompt.txt",
  corpus_path = "tests/fixtures/structured-input-projection/pilot-corpus.json",
  comparator_path = "src/agent_prompt/evaluation.rs",
  compared_profiles = ['json_compact, 'toon_v3],
  retains_raw_prompts = false,
  claims_provider_billing_truth = false,
  claims_production_readiness = false,
} | RuntimeProfile
