# Negative: Refused by the Nickel contract: an unknown field.
let Contract = import "contract.ncl" in
((import "base.ncl") & { network_policy = "bridged" }) | Contract.VmExecutionProfile
