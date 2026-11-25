let Contracts = import "../../../../contracts/credential-providers.ncl" in
let valid = import "../positive/environment.ncl" in
(valid & {
  credential_providers | force = [
    valid.credential_providers[0] & { plaintext = "hostile-secret-provider-value-not-public" },
  ],
}) | Contracts.CredentialAppConfig
