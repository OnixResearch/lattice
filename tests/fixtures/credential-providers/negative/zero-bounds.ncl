let Contracts = import "../../../../contracts/credential-providers.ncl" in
let valid = import "../positive/environment.ncl" in
(valid & {
  credential_providers | force = [
    valid.credential_providers[0] & {
      bounds | force = valid.credential_providers[0].bounds & { timeout_ms | force = 0 },
    },
  ],
}) | Contracts.CredentialAppConfig
