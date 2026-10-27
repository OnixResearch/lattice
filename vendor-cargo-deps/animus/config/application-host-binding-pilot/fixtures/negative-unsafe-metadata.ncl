let profile = import "../profile.ncl" in
{
  expected_issue = "APPLICATION_BINDING_UNSAFE_METADATA",
  fixture | profile.binding_contract = profile.binding & {
    metadata.path | force = "/private/host/path",
  },
}
