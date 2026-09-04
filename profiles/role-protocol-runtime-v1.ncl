# r[impl lattice.role_protocol.runtime_profile]
# r[impl lattice.role_protocol.runtime_profile.scenario.operator]
# r[impl lattice.role_protocol.runtime_profile.scenario.identity]
let Contract = import "../contracts/role-protocol-runtime-profile.ncl" in

({
  schema = "lattice.role-protocol-runtime-profile.v1",
  runtime = {
    repository = "rad:z2yGPFsQZsUcFNkiyR4Dkj22BuxnC",
    revision = "cbb5db0e4844b466dcd1877218eb1f00a2a7caf2",
    source_manifest_identity = "b3:ef80b33109874f6d090efe4791e2980538c5901f4f137ad6d336aa9b675a6a16",
    package = "lattice",
    binary = "lattice",
    command_prefix = ["--config", "__CONFIG_PATH__", "protocol"],
    supported_systems = ["aarch64-linux", "x86_64-linux"],
  },
  service = {
    mode = "operator_oneshot",
    start_at_boot = false,
    restart = "no",
    create_argv = ["create", "__PLAN_PATH__"],
    state_backend = "redb",
    state_directory_binding = "consumer_required",
    service_user_binding = "consumer_required",
  },
  capabilities = {
    operator_commands = ["create", "inspect", "recover", "retry", "abandon", "replay"],
    effect_dispatch = "library_only",
    cli_authority = "unavailable",
    configured_adapters = false,
    automatic_retry = false,
    replay_dispatches_effects = false,
  },
  authority = {
    contract = "lattice-role-protocol",
    evaluation = "current_product",
    credentials = "references_only",
    placement = "consumer_owned",
  },
  required_inputs = [
    "config_path",
    "plan_path",
    "service_user",
    "state_directory",
  ],
  non_claims = [
    "profile admission does not authorize protocol effects",
    "service planning does not prove process execution",
    "runtime placement does not prove target health",
    "operator commands do not provide autonomous dispatch",
    "source linkage does not prove protocol correctness",
    "profile checks do not establish release eligibility",
  ],
}
) | Contract.RoleProtocolRuntimeProfile
