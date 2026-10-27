let profile = import "../profile.ncl" in
{
  expected_issue = "APPLICATION_BINDING_CLAIM_BOUNDARY",
  fixture | profile.binding_contract = profile.binding & {
    semantic_claims | force = ["binding authorizes execution"],
  },
}
