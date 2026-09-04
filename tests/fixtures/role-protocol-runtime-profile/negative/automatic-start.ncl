let Contract = import "../../../../contracts/role-protocol-runtime-profile.ncl" in
let profile = import "../../../../profiles/role-protocol-runtime-v1.ncl" in

(
  profile
  & {
    service.start_at_boot | force = true,
  }
) | Contract.RoleProtocolRuntimeProfile
