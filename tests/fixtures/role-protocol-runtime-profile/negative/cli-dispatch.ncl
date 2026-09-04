let Contract = import "../../../../contracts/role-protocol-runtime-profile.ncl" in
let profile = import "../../../../profiles/role-protocol-runtime-v1.ncl" in

(
  profile
  & {
    capabilities.effect_dispatch | force = "cli",
    capabilities.cli_authority | force = "configured",
    capabilities.configured_adapters | force = true,
  }
) | Contract.RoleProtocolRuntimeProfile
