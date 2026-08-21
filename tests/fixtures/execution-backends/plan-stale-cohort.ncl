let contracts = import "../../../policy/execution-backend-compatibility.ncl" in
(
{
  planner_cohort = "0000000000000000000000000000000000000000",
  planner_schema = "animus.execution-backend-selection.v1",
  planner_plan_identity = "b3:0000000000000000000000000000000000000000000000000000000000000000",
  requirement_profile_identity = "b3:1111111111111111111111111111111111111111111111111111111111111111",
  backend_class = "guarded_local_process",
  backend_identity = "b3:2222222222222222222222222222222222222222222222222222222222222222",
  adapter_identity = "b3:3333333333333333333333333333333333333333333333333333333333333333",
  resources = {
    memory_bytes = 67108864,
    storage_bytes = 4194304,
    cpu_milliseconds = 60000,
    wall_milliseconds = 60000,
  },
}
) | contracts.ProjectionContract
