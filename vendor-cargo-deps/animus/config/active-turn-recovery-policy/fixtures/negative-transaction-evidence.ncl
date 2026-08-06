let schema = import "../../active-turn-recovery-policy.ncl" in
{
  expected_issue = "ACTIVE_TURN_TRANSACTION_EVIDENCE",
  fixture | schema.active_turn_recovery_policy_contract =
    schema.active_turn_recovery_policy & { transaction_evidence_required | force = false },
}
