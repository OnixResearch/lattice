let profile = import "../profile.ncl" in
{
  expected_issue = "APPLICATION_BINDING_EFFECT_CLOSURE",
  fixture | profile.binding_contract = profile.binding & {
    effect_summary_closed | force = false,
  },
}
