let schema = import "../../active-turn-recovery-policy.ncl" in
{
  expected_issue = "ACTIVE_TURN_ENTRY_BOUND",
  fixture | schema.active_turn_recovery_policy_contract =
    schema.active_turn_recovery_policy & { max_entries | force = 0 },
}
