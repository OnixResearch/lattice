let schema = import "../policy.ncl" in
{
  expected_issue = "OK",
  fixture | schema.steel_runtime_policy_contract = schema.steel_runtime_policy,
}
