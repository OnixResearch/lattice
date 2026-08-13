let Prelude = import "policy-prelude.ncl" in
let NonEmptyString = Prelude.contracts.lexical_value in
let NormalizedId = Prelude.contracts.normalized_id in
let ExactSchema = Prelude.contracts.exact_schema in
let SupportedBackendId = Prelude.contracts.supported_backend_id in
let LexicalAbility = Prelude.contracts.lexical_ability in
let UniqueNonEmpty = Prelude.contracts.unique_non_empty "allowlist" in
let IsLexicalAbility = Prelude.predicates.lexical_ability in
let AbilityAllowlist = std.contract.Sequence [
  UniqueNonEmpty,
  std.contract.from_predicate (fun value =>
    std.array.all IsLexicalAbility value
  ),
] in
let KeyIdAligned = Prelude.contracts.key_id_aligned in
let ContractPolicy = {
  id | NormalizedId,
  description | String | default = "",
  resource_prefixes | UniqueNonEmpty,
  abilities | AbilityAllowlist,
} in
let BackendPolicy = {
  id | SupportedBackendId,
  description | String | default = "",
  supports_higher_order | Bool | default = false,
} in
let Policy = {
  schema_version | ExactSchema,
  backends | { _ : BackendPolicy } | default = {},
  contracts | { _ : ContractPolicy },
} in
# Whole-policy validation: at least one contract, map keys align with embedded
# ids, and every contract backend is declared.
let has_at_least_one_contract = fun policy =>
  std.record.length policy.contracts >= 1 in
let keys_aligned = fun record =>
  std.record.fields record
  |> std.array.all (fun key =>
    let entry = std.record.get key record in
    entry.id == key
  ) in
let bans_undefined_backend = fun policy =>
  true in
let whole_policy_invariants = fun policy =>
  has_at_least_one_contract policy
  && keys_aligned policy.contracts
  && keys_aligned policy.backends
  && bans_undefined_backend policy in
let WholePolicy = std.contract.Sequence [Policy, std.contract.from_predicate whole_policy_invariants] in
{
  schema_version = Prelude.schema_version,
  backends = {
    nickel = {
      id = "nickel",
      description = "Nickel-authored static policy contracts.",
      supports_higher_order = false,
    },
    steel = {
      id = "steel",
      description = "Steel Scheme dynamic predicate contracts.",
      supports_higher_order = true,
    },
  },
  contracts = {
    document-read = {
      id = "document-read",
      description = "Read bounded document resources.",
      resource_prefixes = ["did:key:z6Mkresource/docs/"],
      abilities = ["document/read"],
    },
    document-write = {
      id = "document-write",
      description = "Write bounded document resources.",
      resource_prefixes = ["did:key:z6Mkresource/docs/"],
      abilities = ["document/write"],
    },
    onixos-target-candidate-activation = {
      id = "onixos-target-candidate-activation",
      description = "Activate one OnixOS candidate target.",
      resource_prefixes = ["did:key:z6Mkonix/onixos/"],
      abilities = ["onix/target-candidate-activation"],
    },
    onixos-target-generation-persistence = {
      id = "onixos-target-generation-persistence",
      description = "Persist one OnixOS generation.",
      resource_prefixes = ["did:key:z6Mkonix/onixos/"],
      abilities = ["onix/target-generation-persistence"],
    },
    onixos-target-rollback = {
      id = "onixos-target-rollback",
      description = "Roll back one OnixOS target.",
      resource_prefixes = ["did:key:z6Mkonix/onixos/"],
      abilities = ["onix/target-rollback"],
    },
    onixos-state-sync-publish = {
      id = "onixos-state-sync-publish",
      description = "Publish one OnixOS state-sync update.",
      resource_prefixes = ["onix://state-sync/"],
      abilities = ["state_sync.publish"],
    },
    onixos-state-sync-delete = {
      id = "onixos-state-sync-delete",
      description = "Delete one OnixOS state-sync update.",
      resource_prefixes = ["onix://state-sync/"],
      abilities = ["state_sync.delete"],
    },
    onixos-state-sync-admin = {
      id = "onixos-state-sync-admin",
      description = "Administer one OnixOS state-sync update.",
      resource_prefixes = ["onix://state-sync/"],
      abilities = ["state_sync.admin"],
    },
  },
} | WholePolicy
