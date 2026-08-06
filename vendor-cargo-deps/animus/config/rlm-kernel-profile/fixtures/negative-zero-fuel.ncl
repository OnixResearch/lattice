let schema = import "../../rlm-kernel-profile.ncl" in
{
  expected_issue = "RLM_KERNEL_FUEL_BOUND",
  fixture | schema.rlm_kernel_policy_contract =
    schema.rlm_kernel_policy & { bounds.max_evaluation_fuel | force = 0 },
}
