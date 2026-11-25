# Example typed AppConfig snippet configuring an `animus-secretspec` SOPS provider.
# Keep all secret values, age keys, and plaintext payloads out of this file;
# supply only public provider metadata and explicit operator-owned host paths.
let Contracts = import "../../contracts/credential-providers.ncl" in
(
{
  data_dir = "~/.lattice",
  default_timeout_secs = 60,
  shell = "sh",
  credential_providers = [
    {
      kind = "animus-secretspec",
      id = "reviewed-sops",
      cohort = "0.17.0",
      manifest_identity = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      policy_identity = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210",
      profile = "service",
      scope = "workflow",
      reason = "workflow-authentication",
      allowed_schemes = ["sops"],
      declarations = [
        {
          key = "api-token",
          declaration = "API_TOKEN",
          scheme = "sops",
        },
      ],
      bounds = {
        input_max_bytes = 4096,
        value_max_bytes = 65536,
        stdout_max_bytes = 65536 + 32,
        stderr_max_bytes = 4096,
        timeout_ms = 10000,
        poll_interval_ms = 10,
        teardown_timeout_ms = 1000,
      },
    },
  ],
  credential_hosts = [
    {
      provider_id = "reviewed-sops",
      policy_path = "/etc/lattice/secretspec/policy.json",
      manifest_path = "/etc/lattice/secretspec/secretspec.toml",
      worker_executable = "/run/current-system/sw/bin/animus-secretspec-worker",
      working_directory = "/var/lib/lattice/secretspec",
      input = {
        kind = "sops",
        guard_directory = "/etc/lattice/secretspec/sops-guard",
        executable = "/run/current-system/sw/bin/sops",
        credential_variables = [
          {
            credential = "age-key",
            variable = "LATTICE_SOPS_AGE_KEY",
          },
        ],
      },
    },
  ],
}
) | Contracts.CredentialAppConfig
