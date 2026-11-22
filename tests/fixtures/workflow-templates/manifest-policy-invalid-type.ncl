let contract = import "../../../policy/workflow-template-bundle.ncl" in
let base = import "manifest-base.ncl" in
let open_type = {
  name = "command",
  type = { kind = "string", max_bytes = 64 },
  required = true,
  bindings = [{ node = "read input", shell_env = "LATTICE_PARAM_COMMAND" }],
} in
(std.record.update "params" [open_type] base) | contract
