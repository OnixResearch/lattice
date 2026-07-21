let schema = import "../catalog.ncl" in
let base = schema.sample_catalog in
let descriptor = std.array.first base.descriptors in
{
  expected_issue = "TOOL_CATALOG_SCHEMA_REFERENCE",
  fixture | schema.tool_catalog_contract =
    base & { descriptors | force = [descriptor & { input_schema | force = "raw:any" }] },
}
