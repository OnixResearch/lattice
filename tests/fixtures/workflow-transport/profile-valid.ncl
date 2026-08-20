let Contract = import "../../../contracts/workflow-transport-profile.ncl" in
let FrameBytes = 1048576 in
let LogBytes = 65536 in
let Connections = 8 in
let DeadlineMilliseconds = 30000 in
let profile | Contract.TransportProfile = {
  schema = "lattice.workflow-transport-profile.v1",
  limits = {
    maximum_frame_bytes = FrameBytes,
    maximum_log_bytes = LogBytes,
    maximum_connections = Connections,
    read_deadline_milliseconds = DeadlineMilliseconds,
    write_deadline_milliseconds = DeadlineMilliseconds,
  },
  peers = {
    allowed_user_ids = [1000],
  },
  non_claims = [
    "transport admission does not prove workflow correctness",
    "transport peer identity does not prove peer intent",
    "transport acceptance does not prove an operation executed",
    "transport response does not prove an external effect succeeded",
    "transport checks do not establish production deployment",
  ],
} in
profile
