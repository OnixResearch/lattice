let schema = import "../../mcp-client-host.ncl" in
{
  expected_issue = "MCP_TRANSPORT",
  fixture | schema.mcp_client_host_contract = schema.mcp_client_host & {
    server.transport | force = "streamable_http",
  },
}
