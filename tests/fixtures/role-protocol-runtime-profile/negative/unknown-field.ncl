let Contract = import "../../../../contracts/role-protocol-runtime-profile.ncl" in
let profile = import "../../../../profiles/role-protocol-runtime-v1.ncl" in

(
  profile
  & {
    deployment_authority = true,
  }
) | Contract.RoleProtocolRuntimeProfile
