# Synthetic: a worker that needs one directly allocated chip.
let Contract = import "../../../../../contracts/worker-execution-binding.ncl" in
{
  schema = "lattice.worker-execution-binding.v1",
  worker_compute = {
    kind = "provider_execution",
    direct_accelerator = {
      min_chips = 1,
      accepted_granularities = ["mmio-bundle", "chip"],
      backend = "synthetic-backend",
      runtime = "cuda",
      architecture = null,
      minimum_enforcement = "backend-acknowledged",
    },
  },
  worker_model = { kind = "no_model" },
  execution_binding = {
    schema = "lattice.execution-binding.v1",
    compute = {
      kind = "provider_allocation",
      allocation_id = "agent-a-device",
      host_id = "host-a",
      consumer_id = "agent-device",
      owner_generation = 1,
    },
    model_routes = { kind = "no_model" },
  },
} | Contract.WorkerExecutionBinding
