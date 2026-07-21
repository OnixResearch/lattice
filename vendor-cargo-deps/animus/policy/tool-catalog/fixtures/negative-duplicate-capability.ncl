let schema = import "../catalog.ncl" in
let base = schema.sample_catalog in
let descriptor = std.array.first base.descriptors in
{
  expected_issue = "TOOL_CATALOG_DESCRIPTOR_INVARIANT",
  fixture | schema.tool_catalog_contract =
    base & { descriptors | force = [descriptor & { capabilities | force = ["observe_read", "observe_read"] }] },
}
