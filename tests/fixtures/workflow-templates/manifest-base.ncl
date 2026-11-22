# Schema 2 manifest values shared by the authoring-contract fixtures.
{
  schema_version = 2,
  id = "policy-fixture",
  display_name = "Policy fixture",
  description = "Authoring-contract fixture",
  category = "tests",
  mode = "read_only",
  params = [
    {
      name = "root",
      type = { kind = "relative_path", max_bytes = 64 },
      required = false,
      default = ".",
      bindings = [{ node = "read input", shell_env = "LATTICE_PARAM_ROOT" }],
    },
    {
      name = "format",
      type = { kind = "enum", values = ["json", "text"] },
      required = true,
      bindings = [{ node = "read input", shell_env = "LATTICE_PARAM_FORMAT" }],
    },
  ],
  workflow = { path = "workflow.ncl", blake3 = "f9f3217fcbe705d00014b4d8b536f3019da77f761d9b65122f68d12c816a8ce7" },
  assets = [],
  compatibility = { workflow_schema = 1, node_kinds = ["shell"], runtime_kinds = ["shell"], feature_ids = [] },
  operator_notes = [],
  non_claims = [],
}
