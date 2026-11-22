# VMADDR_CID_ANY is never an admissible peer.
let base = import "../profile-v2-vsock.ncl" in
let Contract = import "../../../../contracts/workflow-exchange-handler-profile.ncl" in
let wildcard = std.record.update "peers" { allowed_cids = [2, 4294967295] } base.listener.transport in
(std.record.update "listener" { kind = "vsock", transport = wildcard } base) | Contract.HandlerProfileV2
