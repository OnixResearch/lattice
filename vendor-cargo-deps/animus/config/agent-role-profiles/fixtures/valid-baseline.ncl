let schema = import "../../agent-role-profiles.ncl" in
{
  fixture | schema.agent_role_catalog_contract = schema.agent_role_catalog,
}
