let profile = import "../profile.ncl" in
let base = profile.binding in
{
  expected_issue = "APPLICATION_BINDING_AMBIGUOUS_HOST",
  fixture | profile.binding_contract = base & {
    alternative_host_profile_identities | force = [base.host_profile_identity],
  },
}
