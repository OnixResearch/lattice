let Contracts = import "../../../policy/operation-data.ncl" in
({
  schema = "lattice.operation-data-query.v1",
  version_identity = "b3:not-canonical",
  order = "version_descending",
  maximum_result_count = 1,
} | Contracts.Query)
