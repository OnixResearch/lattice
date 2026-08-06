let schema = import "../../rlm-kernel-profile.ncl" in
{
  expected_issue = "RLM_KERNEL_MISSING_NON_CLAIM",
  fixture | schema.rlm_kernel_policy_contract = schema.rlm_kernel_policy & {
    non_claims | force = [
      "security_sandboxing",
      "model_correctness",
      "task_completion",
      "production_readiness",
    ]
  },
}
