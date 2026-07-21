let contracts = import "../primitives.ncl" in
let UniqueArray = contracts.stable_validator "NICKEL_POLICY_UNIQUE" contracts.all_unique in
{
  expected_issue = "NICKEL_POLICY_UNIQUE",
  fixture | UniqueArray = ["duplicate", "duplicate"],
}
