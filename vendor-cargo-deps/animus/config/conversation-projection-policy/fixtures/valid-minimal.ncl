let schema = import "../../conversation-projection-policy.ncl" in
{ expected_issue = "OK", fixture | schema.conversation_projection_policy_contract = schema.conversation_projection_policy }
