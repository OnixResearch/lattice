let schema = import "../../durable-session-store-policy.ncl" in
{
  expected_issue = "DURABLE_SESSION_STORE_RECEIPT_FIELDS",
  fixture | schema.durable_session_store_policy_contract =
    schema.durable_session_store_policy & { receipt_fields | force = ["store_identity"] },
}
