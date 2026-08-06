let schema = import "../../rlm-kernel-profile.ncl" in
{
  expected_issue = "RLM_KERNEL_OPERATION_CLASS",
  fixture | schema.rlm_kernel_policy_contract =
    schema.rlm_kernel_policy & { admitted_operations | force = ["network"] },
}
