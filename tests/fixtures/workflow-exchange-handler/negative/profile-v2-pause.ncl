# Version two keeps workflow pause and resume refused, whatever the VMM can do.
let base = import "../profile-v2-vsock.ncl" in
let Contract = import "../../../../contracts/workflow-exchange-handler-profile.ncl" in
(std.record.update "controls" { cancel = true, pause = true } base) | Contract.HandlerProfileV2
