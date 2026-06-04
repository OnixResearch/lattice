# CSV cleanup workflow example.
# Pipeline: source CSV → filter active records → sort → unique → render report.

let Port = { id | Number, label | String, type_tag | String } in
let Node = { id | Number, label | String, x | Number, y | Number, input_ports | Array Port, output_ports | Array Port } in
let Edge = { source | Number, target | Number } in
let EnvVar = { key | String, value | String } in
let SecretEnvVar = { name | String, provider | String, key | String, field | Array String } in
let NodeConfig = { node_id | Number, kind | String, command | String, cwd | Array String, env | Array EnvVar, secret_env | Array SecretEnvVar, http_method | String, url | String, headers | Array EnvVar, body | Array String, file_operation | String, file_path | String, file_content | Array String, overwrite | Bool, allowed_roots | Array String, timeout_secs | Number, pipe_stdin | Bool } in
let Meta = { name | String, description | String, created | String, modified | String } in
let Workflow = { meta | Meta, nodes | Array Node, edges | Array Edge, configs | Array NodeConfig } in
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
let shell_config = fun config_node_id config_command => shell_defaults & { node_id = config_node_id, command = config_command } in
(
{
  meta = {
    name = "csv-cleanup-report",
    description = "Clean a CSV stream and render an active-user report",
    created = "2026-06-03T00:00:00Z",
    modified = "2026-06-03T00:00:00Z",
  },
  nodes = [
    {
      id = 1,
      label = "csv source",
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
    {
      id = 2,
      label = "active team users",
      x = 28,
      y = 0,
      input_ports = [
        { id = 20, label = "stdin", type_tag = "string" },
      ],
      output_ports = [
        { id = 21, label = "stdout", type_tag = "string" },
        { id = 22, label = "stderr", type_tag = "string" },
        { id = 23, label = "exit",   type_tag = "exitcode" },
      ],
    },
    {
      id = 3,
      label = "sort rows",
      x = 56,
      y = 0,
      input_ports = [
        { id = 30, label = "stdin", type_tag = "string" },
      ],
      output_ports = [
        { id = 31, label = "stdout", type_tag = "string" },
        { id = 32, label = "stderr", type_tag = "string" },
        { id = 33, label = "exit",   type_tag = "exitcode" },
      ],
    },
    {
      id = 4,
      label = "dedupe rows",
      x = 84,
      y = 0,
      input_ports = [
        { id = 40, label = "stdin", type_tag = "string" },
      ],
      output_ports = [
        { id = 41, label = "stdout", type_tag = "string" },
        { id = 42, label = "stderr", type_tag = "string" },
        { id = 43, label = "exit",   type_tag = "exitcode" },
      ],
    },
    {
      id = 5,
      label = "render csv report",
      x = 112,
      y = 0,
      input_ports = [
        { id = 50, label = "stdin", type_tag = "string" },
      ],
      output_ports = [
        { id = 51, label = "stdout", type_tag = "string" },
        { id = 52, label = "stderr", type_tag = "string" },
        { id = 53, label = "exit",   type_tag = "exitcode" },
      ],
    },
  ],
  edges = [
    { source = 11, target = 20 },
    { source = 21, target = 30 },
    { source = 31, target = 40 },
    { source = 41, target = 50 },
  ],
  configs = [
    shell_config 1 "printf 'name,team,status\nAlice,Ops,active\nBob,Ops,inactive\nAlice,Ops,active\nCara,Data,active\n'",
    shell_config 2 "awk -F ',' 'NR > 1 && $3 == \"active\" { print $2 \",\" $1 }'",
    shell_config 3 "sort",
    shell_config 4 "uniq",
    shell_config 5 "awk -F ',' 'BEGIN { print \"# Active users by team\"; print \"\" } { printf \"- %s: %s\\n\", $1, $2 }'",
  ],
}
) | Workflow
