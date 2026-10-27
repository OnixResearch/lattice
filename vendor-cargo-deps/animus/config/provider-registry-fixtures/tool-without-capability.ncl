let schema = import "../provider-registry.ncl" in
{
  provider_registry | schema.provider_registry_contract = {
    schema = "animus.provider-registry.v1",
    providers = [
      {
        id = "openai",
        credential = {
          source = 'environment,
          label = "ANIMUS_OPENAI_LIVE_API_KEY",
          required = true,
        },
        models = [
          {
            id = "tool-model",
            api_kind = 'openai_responses,
            endpoint = "openai-responses",
            input_modalities = ['text],
            context_budget = 4096,
            output_budget = 1024,
            reasoning = 'optional,
            thinking_levels = ['none, 'low],
            capability_schema = "animus.model-capability-matrix.v1",
            roles = ['user, 'assistant, 'tool],
            content_parts = ['text],
            tool_calls = true,
            structured_outputs = [],
            stream_channels = ['content, 'tool_call, 'usage, 'finish],
            reasoning_controls = ['none, 'low],
            privacy = 'remote_provider,
            cost = {
              input_micro_usd_per_million = 1,
              output_micro_usd_per_million = 1,
            },
          },
        ],
      },
    ],
  },
}
