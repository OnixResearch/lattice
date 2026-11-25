let Contracts = import "../../../../contracts/credential-providers.ncl" in
let valid = import "../positive/environment.ncl" in
(valid & {
  credential_providers | force = [
    valid.credential_providers[0] & { profile | force = "" },
  ],
}) | Contracts.CredentialAppConfig
