# Versioned producer contract for workflow requests and observations.

let Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^b3:[0-9a-f]{64}$" value
) in

let Label = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.length value <= 128
  && std.string.is_match "^[a-z0-9][a-z0-9._:-]*$" value
) in

let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in

let NonNegativeNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value >= 0
) in

let EffectClass = std.contract.from_predicate (fun value =>
  std.is_string value && (
    value == "read"
    || value == "build"
    || value == "test"
    || value == "publish"
    || value == "notify"
    || value == "custom"
  )
) in

let ExecutionState = std.contract.from_predicate (fun value =>
  std.is_string value && (
    value == "accepted"
    || value == "queued"
    || value == "running"
    || value == "paused"
    || value == "completed"
    || value == "failed"
    || value == "cancelled"
    || value == "unknown"
  )
) in

let NodeState = std.contract.from_predicate (fun value =>
  std.is_string value && (
    value == "pending"
    || value == "running"
    || value == "paused"
    || value == "completed"
    || value == "failed"
    || value == "cancelled"
    || value == "skipped"
    || value == "unknown"
  )
) in

let LogStream = std.contract.from_predicate (fun value =>
  std.is_string value && (
    value == "stdout"
    || value == "stderr"
    || value == "diagnostic"
  )
) in

let OptionalIdentity = std.contract.from_predicate (fun value =>
  value == null || (std.is_string value && std.string.is_match "^b3:[0-9a-f]{64}$" value)
) in

let ContractLimits = {
  maximum_nodes | PositiveNumber,
  maximum_edges | PositiveNumber,
  maximum_log_references | PositiveNumber,
  maximum_log_read_bytes | PositiveNumber,
  maximum_total_log_bytes | PositiveNumber,
} in

let ControlCapabilities = {
  cancel | Bool,
  pause | Bool,
  resume | Bool,
} in

let CapabilityProfile = {
  schema | std.contract.from_predicate (fun value => value == "lattice.workflow-profile.v1"),
  graph_schema | std.contract.from_predicate (fun value => value == "lattice.workflow-graph.v1"),
  platforms | Array Label,
  node_kinds | Array Label,
  effect_classes | Array EffectClass,
  adapter_kinds | Array Label,
  controls | ControlCapabilities,
  limits | ContractLimits,
  non_claims | Array String,
} in

let KilnLinkage = {
  effect_identity | Identity,
  attempt_identity | Identity,
  candidate_identity | Identity,
  pipeline_revision_identity | Identity,
  workflow_policy_identity | Identity,
} in

let WorkflowNode = {
  id | Label,
  kind | Label,
  effect_class | EffectClass,
  adapter_kind | Label,
  operation_identity | Identity,
} in

let WorkflowEdge = {
  source | Label,
  target | Label,
} in

let WorkflowGraph = {
  schema | std.contract.from_predicate (fun value => value == "lattice.workflow-graph.v1"),
  identity | Identity,
  nodes | Array WorkflowNode,
  edges | Array WorkflowEdge,
} in

let RequestedControls = {
  cancel | Bool,
  pause | Bool,
  resume | Bool,
} in

let ObservationPolicy = {
  maximum_log_references | PositiveNumber,
  maximum_log_read_bytes | PositiveNumber,
  maximum_total_log_bytes | PositiveNumber,
} in

let WorkflowRequest = {
  schema | std.contract.from_predicate (fun value => value == "lattice.workflow-request.v1"),
  identity | Identity,
  idempotency_key | Identity,
  profile_identity | Identity,
  platform | Label,
  kiln | KilnLinkage,
  graph | WorkflowGraph,
  controls | RequestedControls,
  observation | ObservationPolicy,
} in

let WorkflowAccepted = {
  schema | std.contract.from_predicate (fun value => value == "lattice.workflow-accepted.v1"),
  request_identity | Identity,
  idempotency_key | Identity,
  workflow_identity | Identity,
  execution_identity | Identity,
} in

let NodeObservation = {
  node_id | Label,
  state | NodeState,
  terminal | Bool,
  successful | Bool,
  adapter_identity | Identity,
  receipt_identity | OptionalIdentity,
} in

let LogReference = {
  node_id | Label,
  stream | LogStream,
  reference_identity | Identity,
  cursor | NonNegativeNumber,
  bytes | NonNegativeNumber,
  truncated | Bool,
  gap_before | Bool,
  retained | Bool,
} in

let ExecutionObservation = {
  schema | std.contract.from_predicate (fun value => value == "lattice.workflow-observation.v1"),
  identity | Identity,
  request_identity | Identity,
  idempotency_key | Identity,
  workflow_identity | Identity,
  execution_identity | Identity,
  sequence | NonNegativeNumber,
  state | ExecutionState,
  terminal | Bool,
  successful | Bool,
  nodes | Array NodeObservation,
  logs | Array LogReference,
  receipt_identity | OptionalIdentity,
} in

let ControlAction = std.contract.from_predicate (fun value =>
  std.is_string value && (value == "cancel" || value == "pause" || value == "resume")
) in

let ControlRequest = {
  schema | std.contract.from_predicate (fun value => value == "lattice.workflow-control.v1"),
  request_identity | Identity,
  workflow_identity | Identity,
  execution_identity | Identity,
  expected_sequence | NonNegativeNumber,
  action | ControlAction,
} in

let LogReadRequest = {
  schema | std.contract.from_predicate (fun value => value == "lattice.workflow-log-read.v1"),
  request_identity | Identity,
  workflow_identity | Identity,
  execution_identity | Identity,
  node_id | Label,
  stream | LogStream,
  cursor | NonNegativeNumber,
  maximum_bytes | PositiveNumber,
} in

{
  identity = Identity,
  label = Label,
  capability_profile = CapabilityProfile,
  workflow_request = WorkflowRequest,
  workflow_accepted = WorkflowAccepted,
  execution_observation = ExecutionObservation,
  control_request = ControlRequest,
  log_read_request = LogReadRequest,
}
