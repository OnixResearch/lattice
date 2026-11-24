# Negative fixture: Resource prefix contains non-ASCII Unicode whitespace (U+3000).
let Contracts = import "../../contracts.ncl" in
{
  schema_version = "ucan-nickel-contracts.policy.v1",
  backends = {},
  contracts = {
    document-write = {
      id = "document-write",
      description = "Resource prefix contains non-ASCII Unicode whitespace (U+3000).",
      resource_prefixes = ["did:key:z6Mkresource/　docs/"],
      abilities = ["document/write"],
    },
  },
} | Contracts.Policy
