let schema = import "../../active-turn-recovery-policy.ncl" in
{
  expected_issue = "ACTIVE_TURN_REPLACEMENT_POSTURE",
  fixture | schema.active_turn_recovery_policy_contract =
    schema.active_turn_recovery_policy & { replacement_posture | force = "best_effort" },
}
