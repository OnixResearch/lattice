let schema = import "../../active-turn-recovery-policy.ncl" in
{
  expected_issue = "ACTIVE_TURN_NON_CLAIMS",
  fixture | schema.active_turn_recovery_policy_contract =
    schema.active_turn_recovery_policy & { non_claims | force = ["store_correctness"] },
}
