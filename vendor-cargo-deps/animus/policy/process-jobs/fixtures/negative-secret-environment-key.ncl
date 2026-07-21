let schema = import "../policy.ncl" in
let base = schema.sample_profile_manifest in
let profile = std.array.first base.profiles in
let bad_env = [{ key = "APP_TOKEN", value = "redacted-fixture" }] in
{
  expected_issue = "PROCESS_JOB_PROFILE_INVARIANT",
  fixture | schema.process_job_profile_manifest_contract =
    base & { profiles | force = [profile & { env | force = bad_env }] },
}
