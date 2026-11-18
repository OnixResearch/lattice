# Operator-authored worker compute and model requirements and the per-attempt
# execution-binding declaration. The export decodes into the strict Rust shapes
# in `lattice_core::execution_binding::request` and is revalidated there.
# A declaration expects a provider allocation; it never reserves one.
# r[impl lattice.execution_binding.profile_facts.scenario.unsupported]
let granularities = ["chip", "mmio-bundle", "tray", "mesh", "partition", "alias"] in
let enforcements = ["logical-fence-only", "backend-acknowledged", "backend-fenced"] in

let has_only = fun allowed record => std.array.all (fun field => std.array.elem field allowed) (std.record.fields record) in
let has_all = fun required record => std.array.all (fun field => std.record.has_field field record) required in
let is_shape = fun fields value => std.is_record value && has_only fields value && has_all fields value in
let is_variant = fun value => std.is_record value && std.record.has_field "kind" value in

let is_token = fun value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.length value <= 256
  && !(std.string.is_match "[[:cntrl:]]" value)
in
let is_optional_token = fun value => value == null || is_token value in
let is_reference = fun value => std.is_string value && std.string.is_match "^blake3:[0-9a-f]{64}$" value in
let is_positive = fun value => std.is_number value && std.number.is_integer value && value >= 1 in
let is_distinct = fun values =>
  std.array.length values
  == std.array.length (std.array.fold_left (fun seen value => if std.array.elem value seen then seen else seen @ [value]) [] values)
in
let is_bounded = fun maximum values => std.is_array values && std.array.length values >= 1 && std.array.length values <= maximum in

let is_accelerator = fun value =>
  is_shape ["min_chips", "accepted_granularities", "backend", "runtime", "architecture", "minimum_enforcement"] value
  && is_positive value.min_chips
  && is_bounded 6 value.accepted_granularities
  && std.array.all (fun granularity => std.array.elem granularity granularities) value.accepted_granularities
  && is_distinct value.accepted_granularities
  && is_optional_token value.backend
  && is_optional_token value.runtime
  && is_optional_token value.architecture
  && std.array.elem value.minimum_enforcement enforcements
in

let is_compute_requirement = fun value =>
  is_variant value
  && (
    (value.kind == "host_local" && is_shape ["kind"] value)
    || (
      value.kind == "provider_execution"
      && is_shape ["kind", "direct_accelerator"] value
      && (value.direct_accelerator == null || is_accelerator value.direct_accelerator)
    )
  )
in

let is_route_names = fun names => is_bounded 8 names && std.array.all is_token names && is_distinct names in

let is_model_requirement = fun value =>
  is_variant value
  && (
    (value.kind == "no_model" && is_shape ["kind"] value)
    || (value.kind == "routes" && is_shape ["kind", "names"] value && is_route_names value.names)
  )
in

let is_route = fun route => is_shape ["name", "route_identity"] route && is_token route.name && is_reference route.route_identity in
let is_routes = fun routes =>
  is_bounded 8 routes
  && std.array.all is_route routes
  && is_distinct (std.array.map (fun route => route.name) routes)
in

let is_compute_declaration = fun value =>
  is_variant value
  && (
    (value.kind == "host_local" && is_shape ["kind"] value)
    || (
      value.kind == "provider_allocation"
      && is_shape ["kind", "allocation_id", "host_id", "consumer_id", "owner_generation"] value
      && is_token value.allocation_id
      && is_token value.host_id
      && is_token value.consumer_id
      && is_positive value.owner_generation
    )
  )
in

let is_route_declaration = fun value =>
  is_variant value
  && (
    (value.kind == "no_model" && is_shape ["kind"] value)
    || (value.kind == "routes" && is_shape ["kind", "routes"] value && is_routes value.routes)
  )
in

let is_declaration = fun value =>
  is_shape ["schema", "compute", "model_routes"] value
  && value.schema == "lattice.execution-binding.v1"
  && is_compute_declaration value.compute
  && is_route_declaration value.model_routes
in

# A direct-accelerator or any provider requirement needs a provider placement,
# and declared routes must be exactly the required route names.
let is_consistent = fun value =>
  let requirement = value.worker_compute in
  let declared = value.execution_binding in
  let required_names = if value.worker_model.kind == "routes" then value.worker_model.names else [] in
  let declared_names =
    if declared.model_routes.kind == "routes" then
      std.array.map (fun route => route.name) declared.model_routes.routes
    else
      []
  in
  (requirement.kind == "host_local") == (declared.compute.kind == "host_local")
  && std.array.length required_names == std.array.length declared_names
  && std.array.all (fun name => std.array.elem name declared_names) required_names
in

let is_configuration = fun value =>
  is_shape ["schema", "worker_compute", "worker_model", "execution_binding"] value
  && value.schema == "lattice.worker-execution-binding.v1"
  && is_compute_requirement value.worker_compute
  && is_model_requirement value.worker_model
  && is_declaration value.execution_binding
  && is_consistent value
in

{
  WorkerComputeRequirement = std.contract.from_predicate is_compute_requirement,
  WorkerModelRequirement = std.contract.from_predicate is_model_requirement,
  ExecutionBindingDeclaration = std.contract.from_predicate is_declaration,
  WorkerExecutionBinding = std.contract.from_predicate is_configuration,
}
