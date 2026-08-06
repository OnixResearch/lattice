let schema = import "../../rlm-kernel-profile.ncl" in
{
  expected_issue = "OK",
  fixture | schema.rlm_kernel_policy_contract = schema.rlm_kernel_policy,
}
