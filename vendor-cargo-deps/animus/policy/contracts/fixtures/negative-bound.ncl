let contracts = import "../primitives.ncl" in
{
  expected_issue = "NICKEL_POLICY_BOUNDED_INTEGER",
  fixture | contracts.integer_between "NICKEL_POLICY_BOUNDED_INTEGER" 1 4 = 5,
}
