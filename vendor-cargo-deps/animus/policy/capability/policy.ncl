let GuardedCapability = {
  action | String,
  contract | String,
  resource_prefix | String,
  ability | String,
} in
let CapabilityPolicy = {
  schema | String,
  contracts | Array GuardedCapability,
} in
let policy_schema = "animus.capability-policy.v1" in
let contract_id = "animus.runtime" in
{
  capability_policy | CapabilityPolicy = {
    schema = policy_schema,
    contracts = [
      {
        action = "model",
        contract = contract_id,
        resource_prefix = "model:",
        ability = "call",
      },
      {
        action = "tool",
        contract = contract_id,
        resource_prefix = "tool:",
        ability = "invoke",
      },
      {
        action = "filesystem",
        contract = contract_id,
        resource_prefix = "fs:workspace/",
        ability = "read",
      },
      {
        action = "exec",
        contract = contract_id,
        resource_prefix = "exec:",
        ability = "run",
      },
      {
        action = "network",
        contract = contract_id,
        resource_prefix = "net:",
        ability = "connect",
      },
      {
        action = "steel",
        contract = contract_id,
        resource_prefix = "steel:",
        ability = "evaluate",
      },
      {
        action = "agent_role",
        contract = contract_id,
        resource_prefix = "role:",
        ability = "assign",
      },
      {
        action = "agent_role",
        contract = contract_id,
        resource_prefix = "role:",
        ability = "activate",
      },
      {
        action = "agent_role",
        contract = contract_id,
        resource_prefix = "role:",
        ability = "handoff",
      },
      {
        action = "agent_role",
        contract = contract_id,
        resource_prefix = "role:",
        ability = "emit_candidate",
      },
      {
        action = "agent_role",
        contract = contract_id,
        resource_prefix = "role:",
        ability = "emit_review",
      },
      {
        action = "agent_role",
        contract = contract_id,
        resource_prefix = "role:",
        ability = "emit_verification",
      },
      {
        action = "agent_role",
        contract = contract_id,
        resource_prefix = "role:",
        ability = "emit_judgment",
      },
      {
        action = "agent_role",
        contract = contract_id,
        resource_prefix = "role:",
        ability = "emit_coordination",
      },
      {
        action = "agent_role",
        contract = contract_id,
        resource_prefix = "role:",
        ability = "emit_observation",
      },
      {
        action = "agent_role",
        contract = contract_id,
        resource_prefix = "role:",
        ability = "select",
      },
      {
        action = "agent_role",
        contract = contract_id,
        resource_prefix = "role:",
        ability = "veto",
      },
    ],
  },
}
