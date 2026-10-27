let schema = import "../../mcp-client-host.ncl" in
{
  expected_issue = "MCP_MISSING_NON_CLAIM",
  fixture | schema.mcp_client_host_contract = schema.mcp_client_host & {
    non_claims | force = [
      "server_trust",
      "server_correctness",
      "tool_safety",
      "sandboxing",
      "network_isolation",
      "exactly_once_effects",
      "mcp_parity",
      "production_readiness",
    ],
  },
}
