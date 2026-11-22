# Negative: Refused by the Nickel contract: an unadmitted device.
let Contract = import "contract.ncl" in
(std.record.update "devices" ["vfio-gpu", "virtio-blk-raw", "vsock"] (import "base.ncl")) | Contract.VmExecutionProfile
