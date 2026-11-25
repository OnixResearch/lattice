let Contracts = import "../../../../contracts/credential-providers.ncl" in
let valid = import "../positive/environment.ncl" in
(valid & {
  credential_providers | force = [
    (std.array.at 0 valid.credential_providers) & { plaintext = "hostile-secret-provider-value-not-public" },
  ],
}) | Contracts.CredentialAppConfig
