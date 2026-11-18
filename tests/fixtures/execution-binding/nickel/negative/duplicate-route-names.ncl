# Route names are distinct.
let Contract = import "../../../../../contracts/worker-execution-binding.ncl" in
let route = {
  name = "primary",
  route_identity = "blake3:9999999999999999999999999999999999999999999999999999999999999999",
}
in
{
  schema = "lattice.worker-execution-binding.v1",
  worker_compute = { kind = "host_local" },
  worker_model = { kind = "routes", names = ["primary", "primary"] },
  execution_binding = {
    schema = "lattice.execution-binding.v1",
    compute = { kind = "host_local" },
    model_routes = { kind = "routes", routes = [route, route] },
  },
} | Contract.WorkerExecutionBinding
