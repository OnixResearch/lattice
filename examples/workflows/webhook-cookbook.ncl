# Webhook trigger cookbook workflow.
# Import this workflow, dry-run the listener, then POST JSON to /hooks/cookbook-lead.

let Port = { id | Number, label | String, type_tag | String } in
let Node = { id | Number, label | String, x | Number, y | Number, input_ports | Array Port, output_ports | Array Port } in
let Edge = { source | Number, target | Number } in
let EnvVar = { key | String, value | String } in
let SecretEnvVar = { name | String, provider | String, key | String, field | Array String } in
let NodeConfig = { node_id | Number, kind | String, command | String, cwd | Array String, env | Array EnvVar, secret_env | Array SecretEnvVar, http_method | String, url | String, headers | Array EnvVar, body | Array String, file_operation | String, file_path | String, file_content | Array String, overwrite | Bool, allowed_roots | Array String, timeout_secs | Number, pipe_stdin | Bool } in
let TriggerInputBinding = { input_label | String, source_kind | String, source_name | String, type_tag | String, required | Bool } in
let Trigger = { id | String, workflow_name | String, enabled | Bool, kind | String, webhook_method | String, webhook_path | String, webhook_content_types | Array String, auth_kind | String, auth_provider | String, auth_key | String, auth_field | Array String, auth_header | String, schedule_interval_secs | Number, schedule_next_run_epoch_ms | Number, schedule_missed_run_policy | String, input_bindings | Array TriggerInputBinding, overlap_policy | String, idempotency_policy | String, idempotency_json_pointer | String } in
let Meta = { name | String, description | String, created | String, modified | String } in
let Workflow = { meta | Meta, nodes | Array Node, edges | Array Edge, configs | Array NodeConfig, triggers | Array Trigger } in
let shell_timeout_secs = 60 in
let shell_defaults = {
  kind = "shell",
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
  timeout_secs = shell_timeout_secs,
  pipe_stdin = true,
} in
(
{
  meta = {
    name = "webhook-cookbook",
    description = "Webhook trigger cookbook workflow with credential-reference auth",
    created = "2026-06-07T00:00:00Z",
    modified = "2026-06-07T00:00:00Z",
  },
  nodes = [
    {
      id = 1,
      label = "render webhook receipt",
      x = 0,
      y = 0,
      input_ports = [
        { id = 10, label = "stdin", type_tag = "string" },
      ],
      output_ports = [
        { id = 11, label = "stdout", type_tag = "string" },
        { id = 12, label = "stderr", type_tag = "string" },
        { id = 13, label = "exit",   type_tag = "exitcode" },
      ],
    },
  ],
  edges = [],
  configs = [
    shell_defaults & {
      node_id = 1,
      command = "printf '# Webhook cookbook\n\naccepted trigger: cookbook-lead\n'",
    },
  ],
  triggers = [
    {
      id = "cookbook-lead",
      workflow_name = "webhook-cookbook",
      enabled = true,
      kind = "webhook",
      webhook_method = "POST",
      webhook_path = "/hooks/cookbook-lead",
      webhook_content_types = ["application/json"],
      auth_kind = "bearer",
      auth_provider = "env",
      auth_key = "N8NCLONE_WEBHOOK_TOKEN",
      auth_field = [],
      auth_header = "",
      schedule_interval_secs = 0,
      schedule_next_run_epoch_ms = 0,
      schedule_missed_run_policy = "",
      input_bindings = [
        { input_label = "lead_name", source_kind = "json_pointer", source_name = "/name", type_tag = "json", required = true },
      ],
      overlap_policy = "forbid",
      idempotency_policy = "event_key",
      idempotency_json_pointer = "",
    },
  ],
}
) | Workflow
