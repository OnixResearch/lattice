let schema = import "../policy.ncl" in
let base = schema.sample_profile_manifest in
let profile = std.array.first base.profiles in
let resources = profile.resource_limits & { timeout_seconds | force = 601 } in
{
  expected_issue = "PROCESS_JOB_TIMEOUT",
  fixture | schema.process_job_profile_manifest_contract =
    base & { profiles | force = [profile & { resource_limits | force = resources }] },
}
