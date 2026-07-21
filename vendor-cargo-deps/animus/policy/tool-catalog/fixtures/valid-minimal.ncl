let schema = import "../catalog.ncl" in
{
  expected_issue = "OK",
  fixture | schema.tool_catalog_contract = schema.sample_catalog,
}
