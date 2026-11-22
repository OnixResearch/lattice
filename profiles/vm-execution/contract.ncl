# Structural contract of `lattice.vm-execution-profile.v1` (C1).
# Semantic admission (strictly ascending mobility and devices, no
# `live_precopy` at this revision, `application_checkpoint` needing a
# checkpoint profile, required non-claims) is Lattice's typed admission in
# `src/vm_profile/core.rs`; evaluate, then admit the exported JSON.
# r[impl lattice.vm_profile.admission.scenario.nickel]
let Exact = fun expected => std.contract.from_predicate (fun value => value == expected) in
let OneOf = fun values => std.contract.from_predicate (fun value => std.array.elem value values) in
let Bounded = fun maximum =>
  std.contract.from_predicate (fun value =>
    std.is_number value && std.number.is_integer value && value > 0 && value <= maximum
  )
in
let Port = Bounded 4294967294 in
{
  VmExecutionProfile = {
    schema | Exact "lattice.vm-execution-profile.v1",
    architecture | OneOf ["x86_64", "aarch64"],
    vmm | {
      kind | Exact "cloud-hypervisor",
      version | String,
    },
    guest_manifest_schema | Exact "kiln.lattice-worker-guest.v1",
    guest_network | Exact "none",
    gateway | {
      exchange_vsock_port | Port,
      model_vsock_port | Port,
      host_peer_cid | Exact 2,
      exchange_transport_schema | Exact "lattice.workflow-vsock-transport-profile.v1",
    },
    mobility | Array (OneOf ["pinned", "stopped_transfer", "live_precopy", "application_checkpoint"]),
    checkpoint | OneOf ["none", "workflow-node-boundary-v1"],
    deadline_authority | Exact "host",
    resources | {
      vcpus | Bounded 65535,
      memory_mib | Bounded 4294967295,
      workspace_mib | Bounded 4294967295,
    },
    devices | Array (OneOf ["virtio-blk-raw", "vsock"]),
    non_claims | Array String,
  },
}
