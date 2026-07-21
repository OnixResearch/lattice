let contracts = import "../primitives.ncl" in
{
  expected_issue = "NICKEL_POLICY_SAFE_PATH",
  fixture | contracts.safe_repository_relative_path "NICKEL_POLICY_SAFE_PATH" = "../outside.ncl",
}
