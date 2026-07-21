let schema = import "../../session-tree-export-policy.ncl" in
{
  expected_issue = "SESSION_TREE_EXPORT_ENTRY_BOUND",
  fixture | schema.session_tree_export_policy_contract =
    schema.session_tree_export_policy & { max_entries | force = 0 },
}
