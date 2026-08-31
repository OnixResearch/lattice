let Workflow = {
  name | String,
  enabled | Bool,
  max_nodes | Number,
} in
{
  name = "unknown-field-fixture",
  enabled = true,
  max_nodes = 8,
  unexpected = true,
} | Workflow
