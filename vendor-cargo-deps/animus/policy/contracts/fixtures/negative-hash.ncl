let contracts = import "../primitives.ncl" in
{
  expected_issue = "NICKEL_POLICY_BLAKE3",
  fixture | contracts.lowercase_blake3 "NICKEL_POLICY_BLAKE3" = "ABCDEF",
}
