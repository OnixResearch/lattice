let contracts = import "../../../policy/operation-contracts.ncl" in
(
{
  schema = "lattice.operation-instances.v1",
  instances = [
    {
      id = "unsafe-instance",
      display_alias = "Unsafe behavior override",
      type_id = "cairn.repository",
      type_revision = "b3:04f4f2b5e880435887fd6e8d124b80b9e94c9037ff310aacca88f065d6bf6cc9",
      target = {
        repository_root = "/tmp/repository",
        command = "forbidden",
        api_token = "forbidden-secret-material",
      },
      tags = {},
      method_defaults = {},
      credentials = {},
    },
  ],
}
) | contracts.InstanceFile
