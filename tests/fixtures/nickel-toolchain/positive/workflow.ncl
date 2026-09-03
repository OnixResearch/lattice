let Workflow = {
  name | String,
  enabled | Bool,
  max_nodes | Number,
} in
{
  name = "cohort-fixture",
  enabled = true,
  max_nodes = 8,
} | Workflow
