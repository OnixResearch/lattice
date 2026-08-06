let schema = import "../../active-turn-recovery-policy.ncl" in
{
  expected_issue = "ACTIVE_TURN_CORRUPTION_POSTURE",
  fixture | schema.active_turn_recovery_policy_contract =
    schema.active_turn_recovery_policy & { corruption_posture | force = "skip_rows" },
}
