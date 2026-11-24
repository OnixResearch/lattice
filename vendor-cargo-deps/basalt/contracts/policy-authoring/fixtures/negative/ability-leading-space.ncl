# Negative fixture: Ability contains leading ASCII whitespace.
let Contracts = import "../../contracts.ncl" in
{
  schema_version = "ucan-nickel-contracts.policy.v1",
  backends = {},
  contracts = {
    document-write = {
      id = "document-write",
      description = "Ability contains leading ASCII whitespace.",
      resource_prefixes = ["did:key:z6Mkresource/docs/"],
      abilities = [" document/write"],
    },
  },
} | Contracts.Policy
