# Synthetic: a CPU worker on provider allocation agent-a-1 that names its
# definition and declares task families; it offers no tools.
let Contract = import "../../../../../contracts/worker-execution-binding.ncl" in
{
  schema = "lattice.worker-execution-binding.v1",
  worker_compute = { kind = "provider_execution", direct_accelerator = null },
  worker_model = { kind = "routes", names = ["primary"] },
  execution_binding = {
    schema = "lattice.execution-binding.v1",
    compute = {
      kind = "provider_allocation",
      allocation_id = "agent-a-1",
      host_id = "host-a",
      consumer_id = "agent-1",
      owner_generation = 1,
    },
    model_routes = {
      kind = "routes",
      routes = [
        {
          name = "primary",
          route_identity = "blake3:9999999999999999999999999999999999999999999999999999999999999999",
        },
      ],
    },
  },
  worker_capability = {
    schema = "lattice.worker-capability.v1",
    definition = "rust-implementer",
    task_families = { kind = "declared", names = ["rust-implementation", "cairn-task"] },
    tools = { kind = "unavailable" },
  },
} | Contract.WorkerExecutionBinding
