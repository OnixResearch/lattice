let schema = import "../../agent-check-registry.ncl" in
{
  fixture | schema.agent_check_registry_contract = schema.agent_check_registry,
}
