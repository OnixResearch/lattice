let schema = import "../provider-registry.ncl" in
{
  provider_registry | schema.provider_registry_contract = {
    schema = "animus.provider-registry.v1",
    providers = [
      {
        id = "",
        credential = {
          source = 'none,
          label = "none",
          required = false,
        },
        models = [
          {
            id = "mock-text",
            api_kind = 'mock,
            endpoint = "mock-local",
            input_modalities = ['text],
            context_budget = 4096,
            output_budget = 1024,
            reasoning = 'none,
            thinking_levels = ['none],
            capability_schema = "animus.model-capability-matrix.v1",
            roles = ['user, 'assistant],
            content_parts = ['text],
            tool_calls = false,
            structured_outputs = [],
            stream_channels = [],
            reasoning_controls = ['none],
            privacy = 'local_only,
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
