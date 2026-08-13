let Contract = {
  id | String,
  description | String,
  resource_prefixes | Array String,
  abilities | Array String,
} in
let Policy = {
  schema_version | String,
  contracts | { _ | Contract },
} in
{
  schema_version = "ucan-nickel-contracts.policy.v1",
  contracts = {
    "lattice-file" = {
      id = "lattice-file",
      description = "Admit one capability-relative local file effect.",
      resource_prefixes = ["lattice://file/"],
      abilities = ["file/read", "file/write", "file/append"],
    },
    "lattice-process" = {
      id = "lattice-process",
      description = "Admit one bounded local process effect.",
      resource_prefixes = ["lattice://process/"],
      abilities = ["process/execute"],
    },
    "lattice-network" = {
      id = "lattice-network",
      description = "Admit one selected bounded network effect.",
      resource_prefixes = ["lattice://network/"],
      abilities = ["network/request"],
    },
  },
} | Policy
