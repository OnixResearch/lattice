let schema = import "../catalog.ncl" in
let base = schema.sample_catalog in
let descriptor = std.array.first base.descriptors in
{
  expected_issue = "TOOL_CATALOG_RUNTIME_KIND",
  fixture | schema.tool_catalog_contract =
    base & { descriptors | force = [descriptor & { runtime_kind | force = "plugin" }] },
}
