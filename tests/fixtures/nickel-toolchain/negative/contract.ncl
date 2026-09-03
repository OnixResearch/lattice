let Workflow = {
  name | String,
  enabled | Bool,
  max_nodes | Number,
} in
{
  name = "contract-fixture",
  enabled = "yes",
  max_nodes = 8,
} | Workflow
