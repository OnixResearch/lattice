# Negative: Refused by the Nickel contract: a guest NIC.
let Contract = import "contract.ncl" in
(std.record.update "guest_network" "tap" (import "base.ncl")) | Contract.VmExecutionProfile
