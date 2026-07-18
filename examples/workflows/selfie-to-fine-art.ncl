# Selfie to fine art workflow example.
# Pipeline: selfie + inspiration images -> generic Clankers agent -> generated image.

let Port = { id | Number, label | String, type_tag | String } in
let Node = { id | Number, label | String, x | Number, y | Number, input_ports | Array Port, output_ports | Array Port } in
let Edge = { source | Number, target | Number } in
let EnvVar = { key | String, value | String } in
let SecretEnvVar = { name | String, provider | String, key | String, field | Array String } in
let NodeConfig = { node_id | Number, kind | String, command | String, cwd | Array String, env | Array EnvVar, secret_env | Array SecretEnvVar, http_method | String, url | String, headers | Array EnvVar, body | Array String, file_operation | String, file_path | String, file_content | Array String, overwrite | Bool, allowed_roots | Array String, image_path | String, image_allowed_roots | Array String, image_max_bytes | Number, agent_target | String, agent_prompt | String, agent_prompt_file | Array String, agent_output_mode | String, agent_model_hint | Array String, agent_tool_policy | String, agent_context_refs | Array String, agent_cwd | Array String, agent_session | Array String, timeout_secs | Number, pipe_stdin | Bool } in
let Meta = { name | String, description | String, created | String, modified | String } in
let Workflow = { meta | Meta, nodes | Array Node, edges | Array Edge, configs | Array NodeConfig } in
let image_byte_budget = 10485760 in
let shared_config_defaults = {
  command = "",
  cwd = [],
  env = [],
  secret_env = [],
  http_method = "",
  url = "",
  headers = [],
  body = [],
  file_operation = "",
  file_path = "",
  file_content = [],
  overwrite = false,
  allowed_roots = [],
  timeout_secs = 0,
  pipe_stdin = false,
} in
let image_config = fun config_node_id config_path => shared_config_defaults & {
  node_id = config_node_id,
  kind = "image_input",
  image_path = config_path,
  image_allowed_roots = ["/fixtures"],
  image_max_bytes = image_byte_budget,
  agent_target = "",
  agent_prompt = "",
  agent_prompt_file = [],
  agent_output_mode = "",
  agent_model_hint = [],
  agent_tool_policy = "",
  agent_context_refs = [],
  agent_cwd = [],
  agent_session = [],
} in
let agent_config = fun config_node_id => shared_config_defaults & {
  node_id = config_node_id,
  kind = "agent",
  image_path = "",
  image_allowed_roots = [],
  image_max_bytes = 0,
  agent_target = "clankers",
  agent_prompt = "Convert the selfie into dramatic black-and-white fine art using the inspiration images as style references.",
  agent_prompt_file = [],
  agent_output_mode = "image",
  agent_model_hint = ["image-capable"],
  agent_tool_policy = "image-only",
  agent_context_refs = [],
  agent_cwd = [],
  agent_session = [],
} in
(
{
  meta = {
    name = "selfie-to-fine-art",
    description = "Route a selfie and inspiration images through a generic Clankers agent node",
    created = "2026-06-03T00:00:00Z",
    modified = "2026-06-03T00:00:00Z",
  },
  nodes = [
    {
      id = 1,
      label = "Upload selfie",
      x = 0,
      y = 0,
      input_ports = [],
      output_ports = [
        { id = 11, label = "image", type_tag = "image" },
        { id = 12, label = "metadata", type_tag = "json" },
      ],
    },
    {
      id = 2,
      label = "Inspiration portrait",
      x = 0,
      y = 12,
      input_ports = [],
      output_ports = [
        { id = 21, label = "image", type_tag = "image" },
        { id = 22, label = "metadata", type_tag = "json" },
      ],
    },
    {
      id = 3,
      label = "Inspiration lighting",
      x = 0,
      y = 24,
      input_ports = [],
      output_ports = [
        { id = 31, label = "image", type_tag = "image" },
        { id = 32, label = "metadata", type_tag = "json" },
      ],
    },
    {
      id = 4,
      label = "Fine art agent",
      x = 32,
      y = 8,
      input_ports = [
        { id = 40, label = "prompt", type_tag = "string" },
        { id = 41, label = "selfie", type_tag = "image" },
        { id = 42, label = "inspiration_portrait", type_tag = "image" },
        { id = 43, label = "inspiration_lighting", type_tag = "image" },
        { id = 44, label = "json", type_tag = "json" },
      ],
      output_ports = [
        { id = 45, label = "text", type_tag = "string" },
        { id = 46, label = "json", type_tag = "json" },
        { id = 47, label = "image", type_tag = "image" },
        { id = 48, label = "artifact", type_tag = "bytes" },
        { id = 49, label = "metadata", type_tag = "json" },
      ],
    },
  ],
  edges = [
    { source = 11, target = 41 },
    { source = 21, target = 42 },
    { source = 31, target = 43 },
  ],
  configs = [
    image_config 1 "/fixtures/selfie.png",
    image_config 2 "/fixtures/inspiration-portrait.jpg",
    image_config 3 "/fixtures/inspiration-lighting.png",
    agent_config 4,
  ],
}
) | Workflow
