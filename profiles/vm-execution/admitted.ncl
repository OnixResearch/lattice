# Positive: evaluates to fixtures/vm-profile/execution-profile/admitted.json.
let Contract = import "contract.ncl" in
(import "base.ncl") | Contract.VmExecutionProfile
