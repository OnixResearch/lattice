# Typed application profile for the durable workflow exchange handler.

let Transport = import "workflow-transport-profile.ncl" in
let MaximumRoutes = 1024 in
let MaximumRouteNodes = 4096 in
let MaximumWorkflowNameBytes = 128 in
let MaximumActiveExecutions = 65536 in

let Identity = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^b3:[0-9a-f]{64}$" value
) in

let Label = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.length value <= MaximumWorkflowNameBytes
  && std.string.is_match "^[a-z0-9_.:-]+$" value
) in

let AbsolutePath = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^/[^\\x00]+$" value
) in

let PositiveAtMost = fun maximum => std.contract.from_predicate (fun value =>
  std.is_number value && value > 0 && value <= maximum
) in

let NonNegative = std.contract.from_predicate (fun value =>
  std.is_number value && value >= 0
) in

let ExactSchema = std.contract.from_predicate (fun value =>
  value == "lattice.workflow-exchange-handler-profile.v1"
) in

let EffectAuthority = std.contract.from_predicate (fun value =>
  value == "unavailable" || value == "route_exact"
) in

let RequiredNonClaims = std.contract.from_predicate (fun value =>
  value == [
    "handler admission does not prove workflow correctness",
    "route authority does not prove host confinement",
    "execution observation does not prove an external effect succeeded",
    "peer identity does not prove peer intent",
    "handler checks do not establish release readiness",
  ]
) in

let RouteNode = {
  request_node_id | Label,
  operation_identity | Identity,
  workflow_node_id | NonNegative,
} in

let RouteNodeList = std.contract.from_predicate (fun value =>
  std.is_array value
  && std.array.length value > 0
  && std.array.length value <= MaximumRouteNodes
) in

let Route = {
  profile_identity | Identity,
  graph_identity | Identity,
  workflow_name | Label,
  workflow_revision | Identity,
  effect_authority | EffectAuthority,
  nodes | RouteNodeList | Array RouteNode,
} in

let RouteList = std.contract.from_predicate (fun value =>
  std.is_array value
  && std.array.length value > 0
  && std.array.length value <= MaximumRoutes
) in

{
  HandlerProfile = {
    schema | ExactSchema,
    socket_path | AbsolutePath,
    transport | Transport.TransportProfile,
    maximum_active_executions | PositiveAtMost MaximumActiveExecutions,
    routes | RouteList | Array Route,
    non_claims | RequiredNonClaims,
  },
  RouteNodes = RouteNodeList,
  Routes = RouteList,
}
