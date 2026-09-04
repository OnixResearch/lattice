let Contract = import "../../../../contracts/role-protocol-runtime-profile.ncl" in
let profile = import "../../../../profiles/role-protocol-runtime-v1.ncl" in

(
  profile
  & {
    capabilities.operator_commands | force = [
      "create",
      "inspect",
      "dispatch",
      "recover",
      "retry",
      "abandon",
      "replay",
    ],
  }
) | Contract.RoleProtocolRuntimeProfile
