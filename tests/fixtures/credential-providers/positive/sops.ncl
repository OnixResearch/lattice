let Contracts = import "../../../../contracts/credential-providers.ncl" in
(
{
  data_dir = "/tmp/lattice-secretspec-sops",
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
          key = "token",
          declaration = "API_TOKEN",
          scheme = "sops",
        },
      ],
      bounds = {
        input_max_bytes = 4096,
        value_max_bytes = 1024,
        stdout_max_bytes = 1056,
        stderr_max_bytes = 1024,
        timeout_ms = 5000,
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
      worker_executable = "/usr/bin/animus-secretspec-worker",
      working_directory = "/var/lib/lattice/secretspec",
      input = {
        kind = "sops",
        guard_directory = "/etc/lattice/secretspec/guard",
        executable = "/usr/bin/sops",
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
