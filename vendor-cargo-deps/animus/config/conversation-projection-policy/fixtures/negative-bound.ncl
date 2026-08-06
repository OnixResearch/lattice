let schema = import "../../conversation-projection-policy.ncl" in
{
  expected_issue = "CONVERSATION_PROJECTION_ENTRY_BOUND",
  fixture | schema.conversation_projection_policy_contract = schema.conversation_projection_policy & { max_entries | force = 0 },
}
