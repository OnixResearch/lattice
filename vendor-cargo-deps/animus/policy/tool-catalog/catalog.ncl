# r[impl agent_harness_runtime.nickel_policy_contracts.tool_catalog]
let contracts = import "../contracts/primitives.ncl" in

let catalog_schema_identity = "animus.tool-catalog.schema.v1" in
let catalog_identity = "animus.tool-catalog.v1" in
let maximum_descriptor_count = 256 in
let schema_reference_prefix_literal = "schema:" in
let runtime_kinds = ["host_descriptor", "injected_service", "external_adapter"] in
let capabilities = [
  "observe_read",
  "workspace_mutation",
  "shell",
  "network",
  "process",
  "secret_adjacent",
  "host_custom",
] in
let side_effects = ["read_only", "workspace_write", "process", "network", "secret_adjacent"] in
let approvals = ["allow", "prompt", "deny"] in
let redactions = ["metadata_only", "hash_only", "none"] in
let dangerous_capabilities = ["workspace_mutation", "shell", "network", "process", "secret_adjacent"] in
let built_in_tool_names = ["shell", "network", "model", "tool", "filesystem", "exec"] in

let CatalogSchemaIdentity = contracts.exact "TOOL_CATALOG_SCHEMA_IDENTITY" catalog_schema_identity in
let CatalogIdentity = contracts.exact "TOOL_CATALOG_IDENTITY" catalog_identity in
let ToolName = contracts.safe_identifier "TOOL_CATALOG_NAME" in
let Description = contracts.non_empty_string "TOOL_CATALOG_DESCRIPTION" in
let RuntimeKind = contracts.enum "TOOL_CATALOG_RUNTIME_KIND" runtime_kinds in
let Capability = contracts.enum "TOOL_CATALOG_CAPABILITY" capabilities in
let SideEffect = contracts.enum "TOOL_CATALOG_SIDE_EFFECT" side_effects in
let Approval = contracts.enum "TOOL_CATALOG_APPROVAL" approvals in
let Redaction = contracts.enum "TOOL_CATALOG_REDACTION" redactions in
let SchemaReference = contracts.stable_validator "TOOL_CATALOG_SCHEMA_REFERENCE" (fun value =>
  std.is_string value
  && contracts.starts_with schema_reference_prefix_literal value
  && std.string.is_match "^schema:[a-z0-9][a-z0-9._-]*$" value
) in

let ToolDescriptor = {
  name | ToolName,
  description | Description,
  runtime_kind | RuntimeKind,
  capabilities | Array Capability,
  side_effect | SideEffect,
  approval | Approval,
  input_schema | SchemaReference,
  output_schema | SchemaReference,
  redaction | Redaction,
} in
let ToolCatalogShape = {
  schema | CatalogIdentity,
  descriptors | Array ToolDescriptor,
} in
let ToolCatalogSchemaShape = {
  schema | CatalogSchemaIdentity,
  runtime_kinds | Array RuntimeKind,
  capabilities | Array Capability,
  side_effects | Array SideEffect,
  approvals | Array Approval,
  redactions | Array Redaction,
  schema_reference_prefix | contracts.exact "TOOL_CATALOG_SCHEMA_PREFIX" schema_reference_prefix_literal,
} in

let _has_capability = fun capability descriptor =>
  std.array.elem capability descriptor.capabilities
in

let _descriptor_has_dangerous_capability = fun descriptor =>
  std.array.any (fun capability => std.array.elem capability dangerous_capabilities) descriptor.capabilities
in

let _descriptor_effect_is_consistent = fun descriptor =>
  if descriptor.side_effect == "read_only" then
    !_descriptor_has_dangerous_capability descriptor
  else if descriptor.side_effect == "workspace_write" then
    _has_capability "workspace_mutation" descriptor
  else if descriptor.side_effect == "process" then
    _has_capability "process" descriptor || _has_capability "shell" descriptor
  else if descriptor.side_effect == "network" then
    _has_capability "network" descriptor
  else if descriptor.side_effect == "secret_adjacent" then
    _has_capability "secret_adjacent" descriptor
  else
    false
in

let _descriptor_is_valid = fun descriptor =>
  std.array.length descriptor.capabilities > 0
  && contracts.all_unique descriptor.capabilities
  && !std.array.elem descriptor.name built_in_tool_names
  && _descriptor_effect_is_consistent descriptor
  && (!(_descriptor_has_dangerous_capability descriptor) || descriptor.approval != "allow")
  && (!(_has_capability "secret_adjacent" descriptor) || descriptor.redaction != "none")
in

let _catalog_validator = fun catalog =>
  if std.array.length catalog.descriptors == 0 || std.array.length catalog.descriptors > maximum_descriptor_count then
    'Error { message = "TOOL_CATALOG_DESCRIPTOR_COUNT" }
  else if !contracts.unique_by (fun descriptor => descriptor.name) catalog.descriptors then
    'Error { message = "TOOL_CATALOG_DUPLICATE_NAME" }
  else if !std.array.all _descriptor_is_valid catalog.descriptors then
    'Error { message = "TOOL_CATALOG_DESCRIPTOR_INVARIANT" }
  else
    'Ok
in
let ToolCatalog = contracts.with_validator ToolCatalogShape _catalog_validator in

let _catalog_schema_validator = fun schema =>
  if std.array.length schema.runtime_kinds == 0 || !contracts.all_unique schema.runtime_kinds then
    'Error { message = "TOOL_CATALOG_RUNTIME_VOCABULARY" }
  else if std.array.length schema.capabilities == 0 || !contracts.all_unique schema.capabilities then
    'Error { message = "TOOL_CATALOG_CAPABILITY_VOCABULARY" }
  else if std.array.length schema.side_effects == 0 || !contracts.all_unique schema.side_effects then
    'Error { message = "TOOL_CATALOG_EFFECT_VOCABULARY" }
  else if std.array.length schema.approvals == 0 || !contracts.all_unique schema.approvals then
    'Error { message = "TOOL_CATALOG_APPROVAL_VOCABULARY" }
  else if std.array.length schema.redactions == 0 || !contracts.all_unique schema.redactions then
    'Error { message = "TOOL_CATALOG_REDACTION_VOCABULARY" }
  else
    'Ok
in
let ToolCatalogSchema = contracts.with_validator ToolCatalogSchemaShape _catalog_schema_validator in

let runtime_kind_values = runtime_kinds in
let capability_values = capabilities in
let side_effect_values = side_effects in
let approval_values = approvals in
let redaction_values = redactions in
let schema_reference_prefix_value = schema_reference_prefix_literal in
let catalog_schema_value = {
  schema = catalog_schema_identity,
  runtime_kinds = runtime_kind_values,
  capabilities = capability_values,
  side_effects = side_effect_values,
  approvals = approval_values,
  redactions = redaction_values,
  schema_reference_prefix = schema_reference_prefix_value,
} in
let sample_catalog_value = {
  schema = catalog_identity,
  descriptors = [
    {
      name = "inspect_context",
      description = "Read bounded context supplied by the host.",
      runtime_kind = "host_descriptor",
      capabilities = ["observe_read"],
      side_effect = "read_only",
      approval = "allow",
      input_schema = "schema:animus.tool.input.context.v1",
      output_schema = "schema:animus.tool.output.context.v1",
      redaction = "metadata_only",
    },
  ],
} in
{
  tool_catalog_schema_contract = ToolCatalogSchema,
  tool_catalog_contract = ToolCatalog,
  catalog_schema | ToolCatalogSchema = catalog_schema_value,
  sample_catalog | ToolCatalog = sample_catalog_value,
}
