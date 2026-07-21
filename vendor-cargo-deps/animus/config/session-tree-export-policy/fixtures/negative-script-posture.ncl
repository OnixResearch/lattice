let schema = import "../../session-tree-export-policy.ncl" in
{
  expected_issue = "SESSION_TREE_EXPORT_SCRIPT_POSTURE",
  fixture | schema.session_tree_export_policy_contract =
    schema.session_tree_export_policy & { scripts | force = "allowed" },
}
