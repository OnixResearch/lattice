let contracts = import "../../../policy/operation-contracts.ncl" in
(
{
  schema = "lattice.operation-instances.v1",
  instances = [
    {
      id = "unsafe-instance",
      display_alias = "Unsafe behavior override",
      type_id = "cairn.repository",
      type_revision = "b3:35bb448c27e5def0f6204d8fc3f14fa40275b73a735b1971a1c5954afa30bd2d",
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
