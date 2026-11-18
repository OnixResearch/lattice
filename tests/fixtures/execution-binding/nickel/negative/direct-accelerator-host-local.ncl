# A direct-accelerator requirement cannot be placed host-local.
let Contract = import "../../../../../contracts/worker-execution-binding.ncl" in
{
  schema = "lattice.worker-execution-binding.v1",
  worker_compute = {
    kind = "provider_execution",
    direct_accelerator = {
      min_chips = 1,
      accepted_granularities = ["chip"],
      backend = null,
      runtime = null,
      architecture = null,
      minimum_enforcement = "backend-fenced",
    },
  },
  worker_model = { kind = "no_model" },
  execution_binding = {
    schema = "lattice.execution-binding.v1",
    compute = { kind = "host_local" },
    model_routes = { kind = "no_model" },
  },
} | Contract.WorkerExecutionBinding
