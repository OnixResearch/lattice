let schema = import "../../agent-check-registry.ncl" in
let base = schema.agent_check_registry in
{
  expected_issue = "AGENT_CHECK_DUPLICATE_ID",
  fixture | schema.agent_check_registry_contract = base & { checks | force = base.checks @ [std.array.first base.checks] },
}
