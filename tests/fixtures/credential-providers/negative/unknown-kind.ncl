let Contracts = import "../../../../contracts/credential-providers.ncl" in
let valid = import "../positive/environment.ncl" in
(valid & {
  credential_providers = [
    valid.credential_providers[0] & { kind | force = "direct-sops" },
  ],
}) | Contracts.CredentialAppConfig
