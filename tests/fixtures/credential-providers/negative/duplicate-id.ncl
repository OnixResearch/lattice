let Contracts = import "../../../../contracts/credential-providers.ncl" in
let valid = import "../positive/environment.ncl" in
(valid & {
  credential_providers | force = [
    std.array.at 0 valid.credential_providers,
    std.array.at 0 valid.credential_providers,
  ],
  credential_hosts | force = [
    std.array.at 0 valid.credential_hosts,
    std.array.at 0 valid.credential_hosts,
  ],
}) | Contracts.CredentialAppConfig
