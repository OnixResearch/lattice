let schema = import "../../pueue-host.ncl" in
let base_profile = std.array.first schema.pueue_host.profiles in
let unsafe_profile = base_profile & { command = "echo unsafe" } in
{
  expected_issue = "PUEUE_PROFILE_SHAPE",
  fixture | schema.pueue_host_contract = schema.pueue_host & {
    profiles | force = [unsafe_profile],
  },
}
