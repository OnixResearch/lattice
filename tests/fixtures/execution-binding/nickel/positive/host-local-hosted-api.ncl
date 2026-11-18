# Synthetic: a host-local worker with one brokered hosted-API route; no lease.
let Contract = import "../../../../../contracts/worker-execution-binding.ncl" in
{
  schema = "lattice.worker-execution-binding.v1",
  worker_compute = { kind = "host_local" },
  worker_model = { kind = "routes", names = ["primary"] },
  execution_binding = {
    schema = "lattice.execution-binding.v1",
    compute = { kind = "host_local" },
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
