# Seed runbook-catalog fixture workflow; purpose-made for catalog tests.
{
  meta = {
    name = "hello-runbook",
    description = "Print one fixed runbook line",
    created = "2026-11-21T00:00:00Z",
    modified = "2026-11-21T00:00:00Z",
  },
  nodes = [
    {
      id = 1,
      label = "hello",
      x = 0,
      y = 0,
      input_ports = [{ id = 10, label = "stdin", type_tag = "string" }],
      output_ports = [
        { id = 11, label = "stdout", type_tag = "string" },
        { id = 12, label = "stderr", type_tag = "string" },
        { id = 13, label = "exit", type_tag = "exitcode" },
      ],
    },
  ],
  edges = [],
  configs = [
    {
      node_id = 1,
      kind = "shell",
      command = "printf 'hello from the seed runbook catalog\\n'",
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
      timeout_secs = 60,
      pipe_stdin = true,
    },
  ],
}
