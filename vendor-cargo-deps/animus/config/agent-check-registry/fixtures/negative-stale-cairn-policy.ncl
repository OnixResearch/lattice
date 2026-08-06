let schema = import "../../agent-check-registry.ncl" in
let base = schema.agent_check_registry in
{
  expected_issue = "AGENT_CHECK_CAIRN_POLICY_HASH",
  fixture | schema.agent_check_registry_contract = base & { cairn_policy_hash | force = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" },
}
