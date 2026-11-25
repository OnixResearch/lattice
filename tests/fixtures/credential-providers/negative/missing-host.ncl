let Contracts = import "../../../../contracts/credential-providers.ncl" in
let valid = import "../positive/environment.ncl" in
(valid & {
  credential_hosts | force = [],
}) | Contracts.CredentialAppConfig
