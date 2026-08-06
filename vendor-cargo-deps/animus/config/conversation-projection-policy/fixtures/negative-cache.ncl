let schema = import "../../conversation-projection-policy.ncl" in
{
  expected_issue = "CONVERSATION_PROJECTION_CACHE",
  fixture | schema.conversation_projection_policy_contract = schema.conversation_projection_policy & { cache_posture | force = "authoritative" },
}
