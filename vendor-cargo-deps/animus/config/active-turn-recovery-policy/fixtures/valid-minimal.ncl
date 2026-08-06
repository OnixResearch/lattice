let schema = import "../../active-turn-recovery-policy.ncl" in
{
  expected_issue = "OK",
  fixture | schema.active_turn_recovery_policy_contract = schema.active_turn_recovery_policy,
}
