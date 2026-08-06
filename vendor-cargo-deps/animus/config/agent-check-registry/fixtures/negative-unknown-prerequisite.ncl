let schema = import "../../agent-check-registry.ncl" in
let base = schema.agent_check_registry in
let mutated = std.array.map
  (fun row => if row.check_id == "rust-workspace-clippy" then row & { prerequisites | force = ["missing-check"] } else row)
  base.checks
in
{
  expected_issue = "AGENT_CHECK_UNKNOWN_PREREQUISITE",
  fixture | schema.agent_check_registry_contract = base & { checks | force = mutated },
}
