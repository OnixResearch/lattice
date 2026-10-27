let schema = import "../../mcp-client-host.ncl" in
{
  expected_issue = "MCP_TOOL_COUNT_BOUND",
  fixture | schema.mcp_client_host_contract = schema.mcp_client_host & {
    server.limits.max_tools | force = 0,
  },
}
