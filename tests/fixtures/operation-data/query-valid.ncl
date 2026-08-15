let Contracts = import "../../../policy/operation-data.ncl" in
({
  schema = "lattice.operation-data-query.v1",
  version_identity = "b3:0000000000000000000000000000000000000000000000000000000000000000",
  operation_type = "cairn.repository",
  method = "validate",
  tag = { key = "environment", value = "local" },
  field_path = ["counts", "issues"],
  order = "version_descending",
  maximum_result_count = 1,
} | Contracts.Query)
