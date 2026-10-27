let schema = import "../../mcp-client-host.ncl" in
let overbound_timeout_ms = 300001 in
{
  expected_issue = "MCP_CALL_TIMEOUT",
  fixture | schema.mcp_client_host_contract = schema.mcp_client_host & {
    server.limits.call_timeout_ms | force = overbound_timeout_ms,
  },
}
