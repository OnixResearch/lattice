# Contract Backends Specification Delta

## Purpose

Add portable world-branch authority policy without transferring capability realization or runtime enforcement into Basalt.

## ADDED Requirements

### Requirement: World-branch authority is a pure policy boundary

r[contract_backends.world_branch_authority.boundary] Basalt MUST decide world-branch authority only from supplied normalized requests, reviewed policy, and explicit authority facts. It MUST NOT mint tokens, move ownership, bind adapters, mutate heads, activate branches, promote commits, or execute effects.

#### Scenario: Complete request is evaluated

- GIVEN a caller supplies a complete normalized request and current policy facts
- WHEN Basalt evaluates the request
- THEN it MUST return a bounded decision and obligations without host effects

### Requirement: Policy uses closed branch modes

r[contract_backends.world_branch_authority.policy] Basalt MUST define closed copyable, attenuated, linear, simulation-only, promotion-gated, replace-before-activation, and non-branchable modes. Profiles MUST deny by default and reject unknown or incomplete values.

#### Scenario: Known mode is admitted

- GIVEN a valid rule uses a supported mode and complete normalized fields
- WHEN policy validation runs
- THEN Basalt MUST admit the rule into deterministic evaluation

#### Scenario: Unknown mode is supplied

- GIVEN a rule contains an unsupported mode
- WHEN policy validation runs
- THEN Basalt MUST reject the profile instead of treating the mode as copyable

### Requirement: Decisions reject ambiguity and widening

r[contract_backends.world_branch_authority.decision] Basalt MUST use deterministic rule selection and reject equal-priority conflicts, incompatible obligations, lossy mappings, wider destination limits, and missing required authority facts.

#### Scenario: Exact narrower attenuation passes

- GIVEN one selected rule permits attenuation and the destination limits are no wider than the source limits
- WHEN decision evaluation runs
- THEN Basalt MAY return an attenuated decision with exact obligations

#### Scenario: Destination widens authority

- GIVEN requested destination limits exceed the admitted source limits
- WHEN decision evaluation runs
- THEN Basalt MUST deny the request

### Requirement: Stateful modes bind currentness obligations

r[contract_backends.world_branch_authority.currentness] Linear, promotion-gated, and current-grant decisions MUST bind supplied generation, policy, UCAN, revocation, replay, and scope observations required by the selected profile. Consumers MUST reevaluate before realization.

#### Scenario: Linear facts are current

- GIVEN policy selects linear mode and all required currentness facts are present
- WHEN Basalt constructs the decision
- THEN it MUST require durable source deactivation before destination activation

#### Scenario: Generation fact is stale

- GIVEN the request carries an older durable generation than the current supplied observation
- WHEN evaluation runs
- THEN Basalt MUST deny the request as stale

### Requirement: Consumers retain native realization and enforcement

r[contract_backends.world_branch_authority.consumer] Consumers MUST own capability parsing, token issuance, attenuation proof, durable transfer, secret access, simulation adapters, branch storage, activation, promotion, effect dispatch, observation, and recovery.

#### Scenario: Passing decision lacks realization evidence

- GIVEN Basalt returns a passing mode and obligations
- WHEN the consumer cannot prove those obligations occurred
- THEN branch activation MUST remain denied

### Requirement: Receipts exclude bearer material and enforcement claims

r[contract_backends.world_branch_authority.evidence] Basalt MUST emit metadata-only domain-separated BLAKE3 receipts. They MUST exclude tokens, keys, credentials, raw host paths, secret entropy, private policy bodies, and claims of transfer or enforcement completion.

#### Scenario: Private token is present in caller input

- GIVEN evaluation inputs include bearer material outside the normalized policy request
- WHEN receipt projection runs
- THEN the receipt MUST omit that material

### Requirement: Verification covers every mode and denial boundary

r[contract_backends.world_branch_authority.verification] Basalt MUST test each supported mode and negative unknown, malformed, ambiguous, stale, widening, missing-authority, linear-copy, simulation-escape, promotion-bypass, lossy-mapping, secret, and overclaim cases.

#### Scenario: Focused policy rail runs

- GIVEN Nickel, generated JSON, Rust parity, decisions, receipts, and frozen consumer mappings are current
- WHEN the focused verification rail runs
- THEN it MUST report supported policy facts and runtime-enforcement non-claims
