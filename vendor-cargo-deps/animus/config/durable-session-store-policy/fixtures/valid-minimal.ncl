let schema = import "../../durable-session-store-policy.ncl" in
{
  fixture | schema.durable_session_store_policy_contract = schema.durable_session_store_policy,
}
