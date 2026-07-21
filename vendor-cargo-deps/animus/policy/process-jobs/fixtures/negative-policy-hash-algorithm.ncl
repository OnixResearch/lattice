let schema = import "../policy.ncl" in
{
  expected_issue = "PROCESS_JOB_HASH_ALGORITHM",
  fixture | schema.process_job_profile_policy_contract =
    schema.process_job_profile_policy
    & { identity_hash_algorithm | force = "SHA256" },
}
