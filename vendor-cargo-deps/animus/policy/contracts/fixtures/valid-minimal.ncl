let contracts = import "../primitives.ncl" in
{
  expected_issue = "OK",
  fixture = {
    schema | contracts.exact "NICKEL_POLICY_EXACT_SCHEMA" "animus.example.v1" = "animus.example.v1",
    bounded | contracts.integer_between "NICKEL_POLICY_BOUNDED_INTEGER" 1 4 = 2,
    mode | contracts.enum "NICKEL_POLICY_ENUM" ["allow", "deny"] = "deny",
    identity | contracts.lowercase_blake3 "NICKEL_POLICY_BLAKE3" = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    path | contracts.safe_repository_relative_path "NICKEL_POLICY_SAFE_PATH" = ".cairn/specs/agent-harness-runtime/spec.md",
    unique = contracts.all_unique ["alpha", "beta"],
  },
}
