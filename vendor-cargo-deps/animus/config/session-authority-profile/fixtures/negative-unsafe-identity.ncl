let schema = import "../../session-authority-profile.ncl" in
let base = schema.session_authority_profile in
{
  expected_issue = "SESSION_AUTHORITY_COMPONENT_IDENTITY",
  fixture | schema.session_authority_profile_contract = base & {
    components | force = std.array.map
      (fun component =>
        if component.class == "permission" then
          component & { identity | force = "/home/operator/credential" }
        else
          component)
      base.components,
  },
}
