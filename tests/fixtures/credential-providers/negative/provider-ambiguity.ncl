let Contracts = import "../../../../contracts/credential-providers.ncl" in
let valid = import "../positive/environment.ncl" in
(valid & {
  credential_providers | force = [
    valid.credential_providers[0] & {
      declarations | force = [
        valid.credential_providers[0].declarations[0],
        valid.credential_providers[0].declarations[0] & { declaration | force = "OTHER_TOKEN" },
      ],
    },
  ],
}) | Contracts.CredentialAppConfig
