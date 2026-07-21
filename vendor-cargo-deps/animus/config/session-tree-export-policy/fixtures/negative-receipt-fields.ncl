let schema = import "../../session-tree-export-policy.ncl" in
{
  expected_issue = "SESSION_TREE_EXPORT_RECEIPT_FIELDS",
  fixture | schema.session_tree_export_policy_contract =
    schema.session_tree_export_policy & { receipt_fields | force = ["source_identity"] },
}
