let Contracts = import "../../../../contracts/kiln-workflow-contract.ncl" in
let admitted_edges = 16 in
let admitted_log_references = 8 in
let admitted_log_read_bytes = 4096 in
let admitted_total_log_bytes = 16384 in
({
  schema = "lattice.workflow-profile.v1",
  graph_schema = "lattice.workflow-graph.v1",
  platforms = ["linux-x86_64"],
  node_kinds = ["build"],
  effect_classes = ["build"],
  adapter_kinds = ["local"],
  controls = {
    cancel = true,
    pause = true,
    resume = true,
  },
  limits = {
    maximum_nodes = 0,
    maximum_edges = admitted_edges,
    maximum_log_references = admitted_log_references,
    maximum_log_read_bytes = admitted_log_read_bytes,
    maximum_total_log_bytes = admitted_total_log_bytes,
  },
  non_claims = [],
} | Contracts.capability_profile)
