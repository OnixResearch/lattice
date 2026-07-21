let schema = import "../../durable-session-store-policy.ncl" in
{
  expected_issue = "DURABLE_SESSION_STORE_FRAME_BOUND",
  fixture | schema.durable_session_store_policy_contract =
    schema.durable_session_store_policy & { max_frame_payload_bytes | force = 0 },
}
