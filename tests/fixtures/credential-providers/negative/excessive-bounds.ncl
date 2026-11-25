let Contracts = import "../../../../contracts/credential-providers.ncl" in
let valid = import "../positive/environment.ncl" in
(valid & {
  credential_providers | force = [
    (std.array.at 0 valid.credential_providers) & {
      bounds | force = (std.array.at 0 valid.credential_providers).bounds & { timeout_ms | force = 60001 },
    },
  ],
}) | Contracts.CredentialAppConfig
