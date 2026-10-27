# Typed, non-default SecretSpec host credential-resolution policy.
let NonEmpty = std.contract.from_predicate (fun value => std.is_string value && value != "") in
let Blake3Hex = std.contract.from_predicate (fun value => std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value) in
let Positive = std.contract.from_predicate (fun value => std.is_number value && value > 0) in
let Provider = [| 'environment, 'systemd_credential, 'sops |] in
let Destination = [| 'provider_dispatch, 'service_start, 'continuous_integration |] in
let Bounds = {
  value_count_max | Positive,
  input_max_bytes | Positive,
  value_max_bytes | Positive,
  stdout_max_bytes | Positive,
  stderr_max_bytes | Positive,
  timeout_ms | Positive,
  poll_interval_ms | Positive,
  teardown_timeout_ms | Positive,
} in
let Profile = {
  profile | NonEmpty,
  scope | NonEmpty,
  declaration | NonEmpty,
  manifest_identity | Blake3Hex,
  provider | Provider,
  provider_selector | NonEmpty,
  purpose | NonEmpty,
  destination | Destination,
  bounds | Bounds,
} in
let Policy = {
  schema_version | Positive,
  resolver_kind | NonEmpty,
  ambient | {
    inherit_environment | Bool,
    manifest_discovery | Bool,
  },
  audit | {
    enabled | Bool,
    authoritative | Bool,
  },
  write_operations_allowed | Bool,
  non_claims | Array NonEmpty,
  secretspec | {
    version | NonEmpty,
    commit | NonEmpty,
    crate_checksum | Blake3Hex,
    license | NonEmpty,
    features | Array NonEmpty,
  },
  bounded_exec | {
    commit | NonEmpty,
    license | NonEmpty,
  },
  profiles | Array Profile,
} in
let worker_input_max_bytes = 4096 in
let secret_value_max_bytes = 65536 in
let worker_frame_overhead_bytes = 32 in
let worker_stdout_max_bytes = secret_value_max_bytes + worker_frame_overhead_bytes in
let worker_stderr_max_bytes = 4096 in
let worker_timeout_ms = 10000 in
let worker_poll_interval_ms = 10 in
let worker_teardown_timeout_ms = 1000 in
let common_bounds = {
  value_count_max = 1,
  input_max_bytes = worker_input_max_bytes,
  value_max_bytes = secret_value_max_bytes,
  stdout_max_bytes = worker_stdout_max_bytes,
  stderr_max_bytes = worker_stderr_max_bytes,
  timeout_ms = worker_timeout_ms,
  poll_interval_ms = worker_poll_interval_ms,
  teardown_timeout_ms = worker_teardown_timeout_ms,
} in
let fixture_manifest_identity = "0a647707075f5c8db3e7c3d395ea2ad04416003ee8d988d135551fffc4ef74d4" in
{
  secret_resolution_policy | Policy = {
    schema_version = 1,
    resolver_kind = "secretspec-v0.17.0-bounded-worker-v1",
    ambient = {
      inherit_environment = false,
      manifest_discovery = false,
    },
    audit = {
      enabled = false,
      authoritative = false,
    },
    write_operations_allowed = false,
    non_claims = [
      "caller-authority",
      "credential-correctness",
      "credential-freshness",
      "provider-integrity",
      "audit-completeness",
      "production-readiness",
      "release-eligibility",
    ],
    secretspec = {
      version = "0.17.0",
      commit = "a8794e46ec9664a0e1a3869cc3105d0853937e48",
      crate_checksum = "68498f9695bb3662c157b8fd4b4665a594f1157de022ff5b0f891af4c7ec75d2",
      license = "Apache-2.0",
      features = ["sops"],
    },
    bounded_exec = {
      commit = "29dac88ecded94457572db3fdfaaaab95fa91525",
      license = "AGPL-3.0-or-later",
    },
    profiles = [
      {
        profile = "environment-host",
        scope = "environment-provider",
        declaration = "ENV_API_KEY",
        manifest_identity = fixture_manifest_identity,
        provider = 'environment,
        provider_selector = "env://",
        purpose = "provider_authentication",
        destination = 'provider_dispatch,
        bounds = common_bounds,
      },
      {
        profile = "systemd-host",
        scope = "systemd-provider",
        declaration = "SYSTEMD_API_KEY",
        manifest_identity = fixture_manifest_identity,
        provider = 'systemd_credential,
        provider_selector = "systemd-credential://",
        purpose = "service_start",
        destination = 'service_start,
        bounds = common_bounds,
      },
      {
        profile = "sops-host",
        scope = "sops-provider",
        declaration = "SOPS_API_KEY",
        manifest_identity = fixture_manifest_identity,
        provider = 'sops,
        provider_selector = "sops://encrypted.json?format=json",
        purpose = "provider_authentication",
        destination = 'provider_dispatch,
        bounds = common_bounds,
      },
    ],
  },
}
