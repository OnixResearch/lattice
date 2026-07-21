let schema = import "../../agent-role-profiles.ncl" in
let base = schema.agent_role_catalog in
let mutated_policies = std.array.map
  (fun policy =>
    if policy.policy_id == "independent-review" then
      policy & {
        require_distinct_assignment | force = false,
        require_distinct_actor | force = false,
        require_distinct_run_thread | force = false,
        require_distinct_session | force = false,
        require_distinct_model_profile | force = false,
        require_distinct_workspace | force = false,
        require_distinct_mutation_lineage | force = false,
        require_clean_context | force = false,
        require_subject_immutable | force = false,
        require_verifier_immutable | force = false,
        require_rubric_immutable | force = false,
      }
    else policy)
  base.separation_policies
in
{
  expected_issue = "AGENT_ROLE_SEPARATION_INVARIANT",
  fixture | schema.agent_role_catalog_contract = base & { separation_policies | force = mutated_policies },
}
