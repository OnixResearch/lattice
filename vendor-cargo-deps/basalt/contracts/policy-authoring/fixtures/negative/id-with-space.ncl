# Negative fixture: Contract key and embedded id contain a space.
let Contracts = import "../../contracts.ncl" in
{
  schema_version = "ucan-nickel-contracts.policy.v1",
  backends = {},
  contracts = {
    "document write" = {
      id = "document write",
      description = "Contract key and embedded id contain a space.",
      resource_prefixes = ["did:key:z6Mkresource/docs/"],
      abilities = ["document/write"],
    },
  },
} | Contracts.Policy
