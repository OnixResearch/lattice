let profile = import "../profile.ncl" in
{
  expected_issue = "OK",
  fixture | profile.binding_contract = profile.binding,
}
