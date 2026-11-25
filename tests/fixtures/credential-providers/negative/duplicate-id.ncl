let Contracts = import "../../../../contracts/credential-providers.ncl" in
let valid = import "../positive/environment.ncl" in
(valid & {
  credential_providers | force = [
    valid.credential_providers[0],
    valid.credential_providers[0],
  ],
  credential_hosts | force = [
    valid.credential_hosts[0],
    valid.credential_hosts[0],
  ],
}) | Contracts.CredentialAppConfig
