let contracts = import "../primitives.ncl" in
{
  expected_issue = "NICKEL_POLICY_ENUM",
  fixture | contracts.enum "NICKEL_POLICY_ENUM" ["allow", "deny"] = "ambient",
}
