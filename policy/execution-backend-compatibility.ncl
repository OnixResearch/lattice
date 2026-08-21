# Frozen compatibility contract for the reviewed Animus backend planner cohort.
let PlannerCohort = std.contract.from_predicate (fun value =>
  value == "54b2fdaf57ac9bbc34e6c73f75d803d625f84ec0"
) in
let PlannerSchema = std.contract.from_predicate (fun value =>
  value == "animus.execution-backend-selection.v1"
) in
let IdentityCharacterLength = 67 in
let Identity = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == IdentityCharacterLength
) in
let Resources = {
  memory_bytes | Number,
  storage_bytes | Number,
  cpu_milliseconds | Number,
  wall_milliseconds | Number,
} in
let Projection = {
  planner_cohort | PlannerCohort,
  planner_schema | PlannerSchema,
  planner_plan_identity | Identity,
  requirement_profile_identity | Identity,
  backend_class | String,
  backend_identity | Identity,
  adapter_identity | Identity,
  resources | Resources,
} in
{ ProjectionContract = Projection }
