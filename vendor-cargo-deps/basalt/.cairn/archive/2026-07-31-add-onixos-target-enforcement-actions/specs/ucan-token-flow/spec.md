# Ucan Token Flow Specification Delta

## ADDED Requirements

### Requirement: Closed OnixOS target action contract [r[basalt.ucan_token_flow.onixos_target_actions.contract]]

Basalt MUST provide closed typed actions for candidate activation, generation persistence, rollback, state-sync publish, state-sync delete, and state-sync administration.

#### Scenario: Exact target action binds [r[basalt.ucan_token_flow.onixos_target_actions.contract.exact]]

- GIVEN a Basalt request has the reviewed contract and ability for one target action
- WHEN the action binder evaluates the request
- THEN it MUST produce the operation profile for that exact action
- AND the profile MUST bind the action, contract, resource, and ability

#### Scenario: Target action is substituted [r[basalt.ucan_token_flow.onixos_target_actions.contract.substitution]]

- GIVEN a request or holder signature was prepared for another target action, contract, resource, or ability
- WHEN the adapter evaluates the invocation
- THEN it MUST deny before replay admission
- AND it MUST NOT create live authority

### Requirement: Reviewed target policy mappings [r[basalt.ucan_token_flow.onixos_target_actions.policy]]

Basalt MUST define one reviewed Nickel policy contract for each closed OnixOS target action.

#### Scenario: State-sync authority is attenuated [r[basalt.ucan_token_flow.onixos_target_actions.policy.attenuation]]

- GIVEN a capability grants state-sync publish
- WHEN delete or administrative authority is requested
- THEN Basalt MUST deny the broader action
- AND publish authority MUST remain unchanged

### Requirement: Identity-bound UCAN source output [r[basalt.ucan_token_flow.onixos_target_actions.source]]

The Basalt flake MUST export the checked-in reviewed UCAN snapshot through `lib.ucanSource`.

#### Scenario: Downstream packages UCAN source [r[basalt.ucan_token_flow.onixos_target_actions.source.downstream]]

- GIVEN a downstream consumer pins one immutable Basalt revision
- WHEN it evaluates `basalt.lib.ucanSource`
- THEN the output MUST refer to that revision's checked-in `vendor/ucan` tree
- AND the existing revision, BLAKE3 manifest, and license checks MUST remain authoritative

### Requirement: Positive and negative target-action evidence [r[basalt.ucan_token_flow.onixos_target_actions.tests]]

Tests MUST cover every exact mapping and action substitution before downstream use.

#### Scenario: Complete mapping matrix runs [r[basalt.ucan_token_flow.onixos_target_actions.tests.matrix]]

- GIVEN all six target action variants
- WHEN the focused test matrix runs
- THEN each exact request MUST bind
- AND every mismatched action, contract, ability, or resource prefix MUST fail closed

### Requirement: Bounded target-action documentation [r[basalt.ucan_token_flow.onixos_target_actions.docs]]

Documentation MUST state the exact mappings, source output, and non-claims.

#### Scenario: Reviewer inspects target authority [r[basalt.ucan_token_flow.onixos_target_actions.docs.non_claims]]

- GIVEN a live result admits one target action
- WHEN a reviewer inspects its documented claim
- THEN the claim MUST exclude effect execution, effect correctness, transport security, replay durability, global revocation freshness, and release eligibility

### Requirement: Target-action validation [r[basalt.ucan_token_flow.onixos_target_actions.validation]]

The change MUST pass focused tests, formatting, Clippy, release verification, downstream compatibility, and canonical Cairn gates.

#### Scenario: Validation completes [r[basalt.ucan_token_flow.onixos_target_actions.validation.complete]]

- GIVEN implementation and evidence are complete
- WHEN the validation rail runs
- THEN every required check MUST pass or report one exact bounded blocker

### Requirement: Target-action lifecycle closeout [r[basalt.ucan_token_flow.onixos_target_actions.archive]]

The change MUST sync accepted requirements and archive only after validation passes.

#### Scenario: Change archives [r[basalt.ucan_token_flow.onixos_target_actions.archive.complete]]

- GIVEN all tasks and validation are complete
- WHEN Cairn sync and archive execute
- THEN the accepted specification MUST contain these requirements
- AND the active change MUST be absent
