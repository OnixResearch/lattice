# A worker can expect only an execution allocation, never a serving deployment.
let Contract = import "../../../../../contracts/worker-execution-binding.ncl" in
{
  schema = "lattice.worker-execution-binding.v1",
  worker_compute = { kind = "provider_execution", direct_accelerator = null },
  worker_model = { kind = "no_model" },
  execution_binding = {
    schema = "lattice.execution-binding.v1",
    compute = {
      kind = "serving_deployment",
      allocation_id = "serving-b",
      host_id = "host-b",
      consumer_id = "serving-operator",
      owner_generation = 1,
    },
    model_routes = { kind = "no_model" },
  },
} | Contract.WorkerExecutionBinding
