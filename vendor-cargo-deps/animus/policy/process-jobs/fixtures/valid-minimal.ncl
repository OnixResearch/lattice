let schema = import "../policy.ncl" in
{
  expected_issue = "OK",
  fixture | schema.process_job_profile_manifest_contract = schema.sample_profile_manifest,
}
