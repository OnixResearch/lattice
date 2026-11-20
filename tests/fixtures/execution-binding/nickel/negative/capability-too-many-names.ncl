# A declared set carries at most 32 names.
(import "../capability-base.ncl") { schema = "lattice.worker-capability.v1", definition = "rust-implementer", tools = { kind = "declared", names = std.array.generate (fun index => "tool-%{std.to_string index}") 33 } }
