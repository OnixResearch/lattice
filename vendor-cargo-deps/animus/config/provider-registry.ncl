# Shell-owned provider registry config for Animus.
# The reusable runtime consumes injected model services, not this file.

let NonEmptyString = std.contract.from_predicate (fun value => value != "") in
let PositiveNumber = std.contract.from_predicate (fun value => value > 0) in
let ApiKind = [| 'openai_responses, 'anthropic_messages, 'local_http, 'mock |] in
let Modality = [| 'text, 'image |] in
let ReasoningSupport = [| 'none, 'optional, 'required |] in
let ThinkingLevel = [| 'none, 'low, 'medium, 'high |] in
let Role = [| 'system, 'developer, 'user, 'assistant, 'tool |] in
let ContentPartClass = [| 'text, 'image |] in
let StructuredOutputMode = [| 'json_object, 'json_schema |] in
let StreamChannel = [| 'content, 'tool_call, 'usage |] in
let PrivacyPosture = [| 'local_only, 'private_gateway, 'remote_provider |] in
let CredentialSource = [| 'none, 'environment, 'file, 'oauth |] in
let CostMetadata = {
  input_micro_usd_per_million | PositiveNumber,
  output_micro_usd_per_million | PositiveNumber,
} in
let CredentialConfig = {
  source | CredentialSource,
  label | NonEmptyString,
  required | Bool,
} in
let ModelConfig = {
  id | NonEmptyString,
  api_kind | ApiKind,
  endpoint | NonEmptyString,
  input_modalities | Array Modality,
  context_budget | PositiveNumber,
  output_budget | PositiveNumber,
  reasoning | ReasoningSupport,
  thinking_levels | Array ThinkingLevel,
  capability_schema | NonEmptyString,
  roles | Array Role,
  content_parts | Array ContentPartClass,
  tool_calls | Bool,
  structured_outputs | Array StructuredOutputMode,
  stream_channels | Array StreamChannel,
  reasoning_controls | Array ThinkingLevel,
  privacy | PrivacyPosture,
  cost | CostMetadata,
} in
let ProviderConfig = {
  id | NonEmptyString,
  credential | CredentialConfig,
  models | Array ModelConfig,
} in
let ProviderRegistry = {
  schema | NonEmptyString,
  providers | Array ProviderConfig,
} in

let registry_schema = "animus.provider-registry.v1" in
let default_provider = "mock" in
let default_model = "mock-text" in
let mock_endpoint = "mock-local" in
let text_modality = 'text in
let no_reasoning = 'none in
let no_thinking = 'none in
let mock_api = 'mock in
let no_credential = 'none in
let no_credential_label = "none" in
let default_context_budget = 4096 in
let default_output_budget = 1024 in
let minimum_cost = 1 in
let capability_schema_id = "animus.model-capability-matrix.v1" in
let user_role = 'user in
let assistant_role = 'assistant in
let text_part = 'text in
let local_privacy = 'local_only in
{
  provider_registry_contract = ProviderRegistry,
  provider_registry | ProviderRegistry = {
    schema = registry_schema,
    providers = [
      {
        id = default_provider,
        credential = {
          source = no_credential,
          label = no_credential_label,
          required = false,
        },
        models = [
          {
            id = default_model,
            api_kind = mock_api,
            endpoint = mock_endpoint,
            input_modalities = [text_modality],
            context_budget = default_context_budget,
            output_budget = default_output_budget,
            reasoning = no_reasoning,
            thinking_levels = [no_thinking],
            capability_schema = capability_schema_id,
            roles = [user_role, assistant_role],
            content_parts = [text_part],
            tool_calls = false,
            structured_outputs = [],
            stream_channels = [],
            reasoning_controls = [no_thinking],
            privacy = local_privacy,
            cost = {
              input_micro_usd_per_million = minimum_cost,
              output_micro_usd_per_million = minimum_cost,
            },
          },
        ],
      },
    ],
  },
}
