let Contracts = import "../../../../contracts/credential-providers.ncl" in
let valid = import "../positive/environment.ncl" in
(valid & {
  credential_providers | force = [
    valid.credential_providers[0] & { id | force = "env" },
  ],
  credential_hosts | force = [
    valid.credential_hosts[0] & { provider_id | force = "env" },
  ],
}) | Contracts.CredentialAppConfig
