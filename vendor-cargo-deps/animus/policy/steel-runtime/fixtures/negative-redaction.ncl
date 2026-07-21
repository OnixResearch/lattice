let schema = import "../policy.ncl" in
let base = schema.steel_runtime_policy in
let profile = std.array.first base.profiles in
let redaction = profile.receipt_redaction & { mode | force = "raw" } in
{
  expected_issue = "STEEL_RUNTIME_REDACTION",
  fixture | schema.steel_runtime_policy_contract =
    base & { profiles | force = [profile & { receipt_redaction | force = redaction }] },
}
