# A name cannot carry a command line or uppercase text.
(import "../capability-base.ncl") { schema = "lattice.worker-capability.v1", definition = "rust-implementer", tools = { kind = "declared", names = ["cargo test --all"] } }
