let schema = import "../../session-tree-export-policy.ncl" in
{
  expected_issue = "SESSION_TREE_EXPORT_POLICY_HASH",
  fixture | schema.session_tree_export_policy_contract =
    schema.session_tree_export_policy & { policy_hash | force = "not-blake3" },
}
