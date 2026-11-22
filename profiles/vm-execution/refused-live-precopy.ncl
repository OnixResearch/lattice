# Negative: Evaluates; refused by Lattice admission as `live_precopy_unqualified`.
let Contract = import "contract.ncl" in
(std.record.update "mobility" ["pinned", "live_precopy"] (import "base.ncl")) | Contract.VmExecutionProfile
