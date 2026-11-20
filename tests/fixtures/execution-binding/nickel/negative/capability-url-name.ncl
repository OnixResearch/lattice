# A name cannot carry an endpoint URL.
(import "../capability-base.ncl") { schema = "lattice.worker-capability.v1", definition = "rust-implementer", tools = { kind = "declared", names = ["https://model.invalid/v1"] } }
