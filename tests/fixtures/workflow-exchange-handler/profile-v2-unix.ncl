# Version-two profile that keeps the Unix listener of version one.
let Contract = import "../../../contracts/workflow-exchange-handler-profile.ncl" in
let v1 = import "profile-valid.ncl" in
let profile | Contract.HandlerProfileV2 = {
  schema = "lattice.workflow-exchange-handler-profile.v2",
  listener = {
    kind = "unix",
    socket_path = v1.socket_path,
    transport = v1.transport,
  },
  maximum_active_executions = v1.maximum_active_executions,
  routes = v1.routes,
  non_claims = v1.non_claims,
} in
profile
