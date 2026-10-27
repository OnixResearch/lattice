let schema = import "../../agent-check-registry.ncl" in
let base = schema.agent_check_registry in
{
  expected_issue = "AGENT_CHECK_CAIRN_POLICY_HASH",
  fixture | schema.agent_check_registry_contract = base & {
    cairn_policy_hash | force = "2c0b44345b9d2e2b3eff5fa7db7a14b9681c6bfcf7a4bb651f13bcadc1f91757",
  },
}
