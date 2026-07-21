let schema = import "../catalog.ncl" in
let base = schema.sample_catalog in
let descriptor = std.array.first base.descriptors in
let dangerous = descriptor & {
  capabilities | force = ["process"],
  side_effect | force = "process",
  approval | force = "allow",
} in
{
  expected_issue = "TOOL_CATALOG_DESCRIPTOR_INVARIANT",
  fixture | schema.tool_catalog_contract = base & { descriptors | force = [dangerous] },
}
