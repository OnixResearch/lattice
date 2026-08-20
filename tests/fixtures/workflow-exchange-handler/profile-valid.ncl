let Contract = import "../../../contracts/workflow-exchange-handler-profile.ncl" in
let FrameBytes = 1048576 in
let LogBytes = 65536 in
let Connections = 8 in
let ActiveExecutions = 2 in
let DeadlineMilliseconds = 30000 in
let profile | Contract.HandlerProfile = {
  schema = "lattice.workflow-exchange-handler-profile.v1",
  socket_path = "/run/lattice/workflow-exchange.sock",
  transport = {
    schema = "lattice.workflow-transport-profile.v1",
    limits = {
      maximum_frame_bytes = FrameBytes,
      maximum_log_bytes = LogBytes,
      maximum_connections = Connections,
      read_deadline_milliseconds = DeadlineMilliseconds,
      write_deadline_milliseconds = DeadlineMilliseconds,
    },
    peers = { allowed_user_ids = [1000] },
    non_claims = [
      "transport admission does not prove workflow correctness",
      "transport peer identity does not prove peer intent",
      "transport acceptance does not prove an operation executed",
      "transport response does not prove an external effect succeeded",
      "transport checks do not establish production deployment",
    ],
  },
  maximum_active_executions = ActiveExecutions,
  routes = [{
    profile_identity = "b3:1111111111111111111111111111111111111111111111111111111111111111",
    graph_identity = "b3:2222222222222222222222222222222222222222222222222222222222222222",
    workflow_name = "kiln-ci",
    workflow_revision = "b3:3333333333333333333333333333333333333333333333333333333333333333",
    effect_authority = "route_exact",
    nodes = [{
      request_node_id = "build",
      operation_identity = "b3:4444444444444444444444444444444444444444444444444444444444444444",
      workflow_node_id = 0,
    }],
  }],
  non_claims = [
    "handler admission does not prove workflow correctness",
    "route authority does not prove host confinement",
    "execution observation does not prove an external effect succeeded",
    "peer identity does not prove peer intent",
    "handler checks do not establish release readiness",
  ],
} in
profile
