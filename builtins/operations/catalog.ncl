let contracts = import "../../policy/operation-contracts.ncl" in
(
{
  schema = "lattice.operation-catalog.v2",
  types = [
    {
      id = "cairn.repository",
      schema_version = 2,
      revision = "b3:35bb448c27e5def0f6204d8fc3f14fa40275b73a735b1971a1c5954afa30bd2d",
      description = "Read-only Cairn repository inspection through bounded local process execution",
      tags = ["cairn", "read-only"],
      methods = [
        {
          name = "change_list",
          description = "List the repository change set without mutation",
          arguments = { fields = [], deny_unknown_fields = true },
          output = {
            fields = [
              { name = "changes", value_type = "array", required = true, max_bytes = 65536 },
              { name = "layout", value_type = "string", required = true, max_bytes = 128 },
              { name = "root", value_type = "string", required = true, max_bytes = 4096 },
            ],
            deny_unknown_fields = true,
          },
          preflight = [
            "runtime_available",
            "executable_available",
            "repository_root_admitted",
            "credential_references_valid",
          ],
          capabilities = ["process_read"],
          side_effect = "read_only",
        },
        {
          name = "validate",
          description = "Validate the repository lifecycle package without mutation",
          arguments = { fields = [], deny_unknown_fields = true },
          output = {
            fields = [
              { name = "findings", value_type = "array", required = true, max_bytes = 65536 },
              { name = "issues", value_type = "array", required = true, max_bytes = 65536 },
              { name = "valid", value_type = "boolean", required = true, max_bytes = 16 },
            ],
            deny_unknown_fields = false,
          },
          preflight = [
            "runtime_available",
            "executable_available",
            "repository_root_admitted",
            "credential_references_valid",
          ],
          capabilities = ["process_read"],
          side_effect = "read_only",
        },
      ],
      runtime = {
        kind = "process",
        key = "cairn-read-only-v1",
        executor_identity = "b3:025bc96d707d911025ad5f5026b61c66c7f75c8777f7096a4eb58ed09e4249ac",
      },
      execution_requirements = {
        format = "native_binary",
        filesystem = "read_only_filesystem",
        network = "network_denied",
        process = "single_process",
        terminal = "no_terminal",
        peripheral = "no_peripheral",
        persistence = "ephemeral",
        isolation = "guarded_process_isolation",
        resources = {
          memory_bytes = 67108864,
          storage_bytes = 4194304,
          cpu_milliseconds = 60000,
          wall_milliseconds = 60000,
        },
      },
      capabilities = ["process_read"],
      credential_bindings = [],
      bounds = {
        timeout_seconds = 60,
        input_bytes = 4096,
        output_bytes = 1048576,
        artifact_bytes = 4096,
      },
      required_non_claims = [
        "content identities prove exact bytes only",
        "invocation plans do not grant execution authority",
        "process success does not prove external-system truth",
        "operation results do not prove production or release readiness",
      ],
    },
  ],
}
) | contracts.Catalog
