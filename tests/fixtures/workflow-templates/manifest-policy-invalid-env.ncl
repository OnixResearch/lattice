let contract = import "../../../policy/workflow-template-bundle.ncl" in
let base = import "manifest-base.ncl" in
let ambient = {
  name = "path",
  type = { kind = "relative_path", max_bytes = 64 },
  required = true,
  bindings = [{ node = "read input", shell_env = "PATH" }],
} in
(std.record.update "params" [ambient] base) | contract
