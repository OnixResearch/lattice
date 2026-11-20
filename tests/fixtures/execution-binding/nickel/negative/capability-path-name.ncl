# A name cannot carry a physical path.
(import "../capability-base.ncl") { schema = "lattice.worker-capability.v1", definition = "rust-implementer", tools = { kind = "declared", names = ["/usr/bin/bash"] } }
