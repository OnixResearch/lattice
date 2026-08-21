let Contracts = import "../../../contracts/kiln-workflow-contract.ncl" in
let canonical_identity = "b3:0000000000000000000000000000000000000000000000000000000000000000" in
let admitted_log_references = 8 in
let admitted_log_read_bytes = 4096 in
let admitted_total_log_bytes = 16384 in
({
  schema = "lattice.workflow-request.v2",
  identity = canonical_identity,
  idempotency_key = canonical_identity,
  profile_identity = canonical_identity,
  platform = "linux-x86_64",
  kiln = {
    effect_identity = canonical_identity,
    attempt_identity = canonical_identity,
    candidate_identity = canonical_identity,
    pipeline_revision_identity = canonical_identity,
    workflow_policy_identity = canonical_identity,
  },
  source = {
    provider = "radicle",
    repository = "rad:z2wsvXm5S2sJGvuV1k5JHiwi1PbKE",
    revision = "2dec0fd67328c1eadd2f6c2bab2c71284750fcde",
  },
  graph = {
    schema = "lattice.workflow-graph.v1",
    identity = canonical_identity,
    nodes = [
      {
        id = "build",
        kind = "build",
        effect_class = "build",
        adapter_kind = "local",
        operation_identity = canonical_identity,
      },
    ],
    edges = [],
  },
  controls = {
    cancel = true,
    pause = true,
    resume = true,
  },
  observation = {
    maximum_log_references = admitted_log_references,
    maximum_log_read_bytes = admitted_log_read_bytes,
    maximum_total_log_bytes = admitted_total_log_bytes,
  },
} | Contracts.workflow_request)
