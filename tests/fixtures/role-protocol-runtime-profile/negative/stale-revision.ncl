let Contract = import "../../../../contracts/role-protocol-runtime-profile.ncl" in
let profile = import "../../../../profiles/role-protocol-runtime-v1.ncl" in

(
  profile
  & {
    runtime.revision | force = "0000000000000000000000000000000000000000",
  }
) | Contract.RoleProtocolRuntimeProfile
