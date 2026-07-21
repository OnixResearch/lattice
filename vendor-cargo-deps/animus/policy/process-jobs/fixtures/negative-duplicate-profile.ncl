let schema = import "../policy.ncl" in
let base = schema.sample_profile_manifest in
let profile = std.array.first base.profiles in
{
  expected_issue = "PROCESS_JOB_DUPLICATE_PROFILE",
  fixture | schema.process_job_profile_manifest_contract =
    base & { profiles | force = [profile, profile] },
}
