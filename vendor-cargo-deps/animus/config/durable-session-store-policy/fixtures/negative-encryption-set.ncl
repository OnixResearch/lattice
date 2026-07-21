let schema = import "../../durable-session-store-policy.ncl" in
{
  expected_issue = "DURABLE_SESSION_STORE_ENCRYPTION_SET",
  fixture | schema.durable_session_store_policy_contract =
    schema.durable_session_store_policy & { encryption_postures | force = ["absent_explicit"] },
}
