# Typed application profiles for the durable workflow exchange handler:
# version one (Unix socket) and version two (explicit Unix or vsock listener).

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

let ExactSchemaV2 = std.contract.from_predicate (fun value =>
  value == "lattice.workflow-exchange-handler-profile.v2"
) in

let EffectAuthority = std.contract.from_predicate (fun value =>
  value == "unavailable" || value == "route_exact"
) in

let SourceBinding = std.contract.from_predicate (fun value =>
  value == "unavailable" || value == "shell_environment"
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
  source_binding | SourceBinding,
  nodes | RouteNodeList | Array RouteNode,
} in

let RouteList = std.contract.from_predicate (fun value =>
  std.is_array value
  && std.array.length value > 0
  && std.array.length value <= MaximumRoutes
) in

let HandlerControls = {
  cancel | Bool,
  pause | std.contract.from_predicate (fun value => value == false) | optional,
  resume | std.contract.from_predicate (fun value => value == false) | optional,
} in

let UnixListener = {
  kind | std.contract.from_predicate (fun value => value == "unix"),
  socket_path | AbsolutePath,
  transport | Transport.TransportProfile,
} in

let VsockListener = {
  kind | std.contract.from_predicate (fun value => value == "vsock"),
  transport | Transport.VsockTransportProfile,
} in

# A listener is tagged by `kind`; any tag other than "vsock" must be a Unix listener.
let Listener = std.contract.custom (fun label value =>
  if std.is_record value && std.record.has_field "kind" value && value.kind == "vsock" then
    'Ok (std.contract.apply VsockListener label value)
  else
    'Ok (std.contract.apply UnixListener label value)
) in

{
  HandlerProfile = {
    schema | ExactSchema,
    socket_path | AbsolutePath,
    transport | Transport.TransportProfile,
    maximum_active_executions | PositiveAtMost MaximumActiveExecutions,
    controls | HandlerControls | optional,
    routes | RouteList | Array Route,
    non_claims | RequiredNonClaims,
  },
  HandlerProfileV2 = {
    schema | ExactSchemaV2,
    listener | Listener,
    maximum_active_executions | PositiveAtMost MaximumActiveExecutions,
    controls | HandlerControls | optional,
    routes | RouteList | Array Route,
    non_claims | RequiredNonClaims,
  },
  RouteNodes = RouteNodeList,
  Routes = RouteList,
}
