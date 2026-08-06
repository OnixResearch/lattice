let schema = import "../../session-authority-profile.ncl" in
let base = schema.session_authority_profile in
{
  expected_issue = "SESSION_AUTHORITY_DUPLICATE_COMPONENT",
  fixture | schema.session_authority_profile_contract = base & {
    components | force = std.array.map
      (fun component =>
        if component.class == "adapter_cohort" then
          component & { class | force = "policy_authorization" }
        else
          component)
      base.components,
  },
}
