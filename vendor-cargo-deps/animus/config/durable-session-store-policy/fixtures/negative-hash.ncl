let schema = import "../../durable-session-store-policy.ncl" in
{
  expected_issue = "DURABLE_SESSION_STORE_POLICY_HASH",
  fixture | schema.durable_session_store_policy_contract =
    schema.durable_session_store_policy & { policy_hash | force = "not-blake3" },
}
