let schema = import "../../session-authority-profile.ncl" in
let base = schema.session_authority_profile in
{
  expected_issue = "SESSION_AUTHORITY_MISSING_COMPONENT",
  fixture | schema.session_authority_profile_contract = base & {
    components | force = std.array.filter
      (fun component => component.class != "adapter_cohort")
      base.components,
  },
}
