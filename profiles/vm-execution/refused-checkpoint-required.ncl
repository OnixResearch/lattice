# Negative: Evaluates; refused by Lattice admission as `checkpoint_required`.
let Contract = import "contract.ncl" in
(std.record.update "checkpoint" "none" (import "base.ncl")) | Contract.VmExecutionProfile
