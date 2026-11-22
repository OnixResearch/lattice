# Peer CIDs must be strictly ascending.
let base = import "../profile-v2-vsock.ncl" in
let Contract = import "../../../../contracts/workflow-exchange-handler-profile.ncl" in
let unsorted = std.record.update "peers" { allowed_cids = [3, 2] } base.listener.transport in
(std.record.update "listener" { kind = "vsock", transport = unsorted } base) | Contract.HandlerProfileV2
