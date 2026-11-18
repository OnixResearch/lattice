# Synthetic: a CPU worker on provider allocation agent-a-1 with a separately
# hosted model route. The serving deployment is not declared or reserved here.
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
} | Contract.WorkerExecutionBinding
