# Lattice-owned extension to the reviewed Cairn policy contract.
#
# The producer reads this optional field to project consumer-selected release
# gates; the base upstream contract remains an unchanged snapshot.
{
  Policy = {
    release_readiness_policy | {
      required_external_commands | Array String,
      mcp_completion_contract | String,
      mcp_completion_results | String,
    },
  },
}
