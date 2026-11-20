# A name carries at most 64 bytes.
(import "../capability-base.ncl") { schema = "lattice.worker-capability.v1", definition = "rust-implementer", task_families = { kind = "declared", names = [std.string.join "" (std.array.replicate 65 "a")] } }
