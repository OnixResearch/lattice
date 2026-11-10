# Lattice-owned extension to the reviewed Cairn policy contract.
#
# The producer reads this optional field to project consumer-selected release
# gates; the base upstream contract remains an unchanged snapshot.
let Commands = std.contract.from_predicate (fun values =>
  std.is_array values
  && std.array.length values > 0
  && std.array.all (fun value => std.is_string value && std.string.trim value != "") values
  && std.array.all
    (fun value => std.array.length (std.array.filter (fun other => other == value) values) == 1)
    values
) in
let CompletionContract = std.contract.from_predicate (fun value =>
  std.is_string value && value == "target/release-readiness/mcp-contract.json"
) in
let CompletionResults = std.contract.from_predicate (fun value =>
  std.is_string value && value == "target/release-readiness/mcp-results.json"
) in
{
  Policy = {
    release_readiness_policy | {
      required_external_commands | Commands,
      mcp_completion_contract | CompletionContract,
      mcp_completion_results | CompletionResults,
    },
  },
}
