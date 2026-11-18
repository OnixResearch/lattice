# A worker profile declares at most eight named model routes.
let Contract = import "../../../../../contracts/worker-execution-binding.ncl" in
let names = ["r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8", "r9"] in
{
  schema = "lattice.worker-execution-binding.v1",
  worker_compute = { kind = "host_local" },
  worker_model = { kind = "routes", names = names },
  execution_binding = {
    schema = "lattice.execution-binding.v1",
    compute = { kind = "host_local" },
    model_routes = {
      kind = "routes",
      routes =
        std.array.map
          (fun name => {
            name = name,
            route_identity = "blake3:9999999999999999999999999999999999999999999999999999999999999999",
          })
          names,
    },
  },
} | Contract.WorkerExecutionBinding
