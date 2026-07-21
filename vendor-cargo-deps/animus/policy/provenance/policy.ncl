let MutationTool = {
  tool_name | String,
  mutation_kind | String,
  file_identity_fields | Array String,
  backend_requirement | String,
  receipt_redaction | String,
}
in
let ProvenancePolicy = {
  schema | String,
  canonical_mutation_kinds | Array String,
  backend_requirements | Array String,
  receipt_redactions | Array String,
  tools | Array MutationTool,
}
in
let policy_schema = "animus.provenance.git-ai.policy.v1" in
let default_file_identity_fields = ["relative_path", "stable_id", "content_hash"] in
{
  provenance_policy | ProvenancePolicy = {
    schema = policy_schema,
    canonical_mutation_kinds = ["edit", "write", "replace", "rename"],
    backend_requirements = ["disabled", "optional", "required"],
    receipt_redactions = ["public", "metadata-only", "redacted-payload"],
    tools = [
      {
        tool_name = "edit",
        mutation_kind = "edit",
        file_identity_fields = default_file_identity_fields,
        backend_requirement = "required",
        receipt_redaction = "metadata-only",
      },
      {
        tool_name = "write",
        mutation_kind = "write",
        file_identity_fields = default_file_identity_fields,
        backend_requirement = "required",
        receipt_redaction = "metadata-only",
      },
      {
        tool_name = "replace",
        mutation_kind = "replace",
        file_identity_fields = default_file_identity_fields,
        backend_requirement = "required",
        receipt_redaction = "metadata-only",
      },
      {
        tool_name = "rename",
        mutation_kind = "rename",
        file_identity_fields = ["old_relative_path", "new_relative_path", "stable_id", "content_hash"],
        backend_requirement = "optional",
        receipt_redaction = "metadata-only",
      },
    ],
  },

  invalid_policy_cases = {
    unknown_mutation_kind = {
      tool_name = "commit",
      mutation_kind = "git-commit",
      file_identity_fields = default_file_identity_fields,
      backend_requirement = "required",
      receipt_redaction = "metadata-only",
    },
    missing_file_identity_fields = {
      tool_name = "edit",
      mutation_kind = "edit",
      file_identity_fields = [],
      backend_requirement = "required",
      receipt_redaction = "metadata-only",
    },
    unsupported_backend_requirement = {
      tool_name = "edit",
      mutation_kind = "edit",
      file_identity_fields = default_file_identity_fields,
      backend_requirement = "git-only",
      receipt_redaction = "metadata-only",
    },
    unsafe_receipt_field = {
      tool_name = "edit",
      mutation_kind = "edit",
      file_identity_fields = default_file_identity_fields,
      backend_requirement = "required",
      receipt_redaction = "raw_prompt",
    },
  },

  invalid_type_fixture | ProvenancePolicy = {
    schema = policy_schema,
    canonical_mutation_kinds = ["edit"],
    backend_requirements = ["required"],
    receipt_redactions = ["metadata-only"],
    tools = [
      {
        tool_name = "edit",
        mutation_kind = 1,
        file_identity_fields = default_file_identity_fields,
        backend_requirement = "required",
        receipt_redaction = "metadata-only",
      },
    ],
  },
}
