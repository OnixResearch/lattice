let Contracts = import "../../../../contracts/credential-providers.ncl" in
let valid = import "../positive/environment.ncl" in
(valid & {
  credential_providers | force = [
    (std.array.at 0 valid.credential_providers) & { id | force = "../unsafe\nmetadata" },
  ],
  credential_hosts | force = [
    (std.array.at 0 valid.credential_hosts) & { provider_id | force = "../unsafe\nmetadata" },
  ],
}) | Contracts.CredentialAppConfig
