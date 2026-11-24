# Negative fixture: Ability contains a newline control character.
let Contracts = import "../../contracts.ncl" in
{
  schema_version = "ucan-nickel-contracts.policy.v1",
  backends = {},
  contracts = {
    document-write = {
      id = "document-write",
      description = "Ability contains a newline control character.",
      resource_prefixes = ["did:key:z6Mkresource/docs/"],
      abilities = ["document/\nwrite"],
    },
  },
} | Contracts.Policy
