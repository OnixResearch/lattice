# Immutable consumer profile for the reviewed projected-role operator runtime.
# r[impl lattice.role_protocol.runtime_profile]
# r[impl lattice.role_protocol.runtime_profile.scenario.unsupported]
# r[impl lattice.role_protocol.runtime_profile.scenario.authority]
let RuntimeRevision = "cbb5db0e4844b466dcd1877218eb1f00a2a7caf2" in
let SourceManifest = "b3:ef80b33109874f6d090efe4791e2980538c5901f4f137ad6d336aa9b675a6a16" in

let Exact = fun expected => std.contract.from_predicate (fun value => value == expected) in

let Runtime = {
  repository | Exact "rad:z2yGPFsQZsUcFNkiyR4Dkj22BuxnC",
  revision | Exact RuntimeRevision,
  source_manifest_identity | Exact SourceManifest,
  package | Exact "lattice",
  binary | Exact "lattice",
  command_prefix | Exact ["--config", "__CONFIG_PATH__", "protocol"],
  supported_systems | Exact ["aarch64-linux", "x86_64-linux"],
}
in

let Service = {
  mode | Exact "operator_oneshot",
  start_at_boot | Exact false,
  restart | Exact "no",
  create_argv | Exact ["create", "__PLAN_PATH__"],
  state_backend | Exact "redb",
  state_directory_binding | Exact "consumer_required",
  service_user_binding | Exact "consumer_required",
}
in

let Capabilities = {
  operator_commands | Exact ["create", "inspect", "recover", "retry", "abandon", "replay"],
  effect_dispatch | Exact "library_only",
  cli_authority | Exact "unavailable",
  configured_adapters | Exact false,
  automatic_retry | Exact false,
  replay_dispatches_effects | Exact false,
}
in

let Authority = {
  contract | Exact "lattice-role-protocol",
  evaluation | Exact "current_product",
  credentials | Exact "references_only",
  placement | Exact "consumer_owned",
}
in

let RequiredInputs =
  Exact [
    "config_path",
    "plan_path",
    "service_user",
    "state_directory",
  ]
in

let RequiredNonClaims =
  Exact [
    "profile admission does not authorize protocol effects",
    "service planning does not prove process execution",
    "runtime placement does not prove target health",
    "operator commands do not provide autonomous dispatch",
    "source linkage does not prove protocol correctness",
    "profile checks do not establish release eligibility",
  ]
in

{
  RoleProtocolRuntimeProfile = {
    schema | Exact "lattice.role-protocol-runtime-profile.v1",
    runtime | Runtime,
    service | Service,
    capabilities | Capabilities,
    authority | Authority,
    required_inputs | RequiredInputs,
    non_claims | RequiredNonClaims,
  },
}
