# Runtime Effect Permission Policy Delta

## ADDED Requirements

### Requirement: Runtime effect permissions are portable policy facts

r[contract_backends.runtime_effect_permissions.boundary] Basalt MUST define runtime effect permissions as deterministic decisions over supplied normalized requests, reviewed policy, and explicit authority facts. The core MUST NOT read files, open networks, inspect environments, spawn processes, invoke bindings, or execute allowed effects.

#### Scenario: Normalized request is evaluated

- GIVEN a consumer supplies a complete normalized effect request and current policy facts
- WHEN Basalt evaluates the request
- THEN it returns a bounded allow or deny decision without host effects

### Requirement: Effect policy uses closed scopes and operations

r[contract_backends.runtime_effect_permissions.policy] Basalt MUST define versioned filesystem, network, process, environment, and binding scopes with closed operation vocabularies. Profiles MUST declare a default decision, rule match mode, subject pattern, explicit priority, policy identity, and non-claims.

#### Scenario: Known scope is admitted

- GIVEN a profile contains a supported scope, operation, match mode, and normalized subject
- WHEN policy validation runs
- THEN Basalt admits the rule into deterministic evaluation

#### Scenario: Unknown operation fails closed

- GIVEN a profile contains an unsupported scope, operation, or match mode
- WHEN policy validation runs
- THEN Basalt denies the profile instead of ignoring the unknown value

### Requirement: Secure profiles deny by default and reject ambiguity

r[contract_backends.runtime_effect_permissions.decision] Basalt MUST support exact and declared prefix matching with explicit priorities. The secure baseline MUST deny by default. Equal-priority conflicting matches, malformed subjects, and incomplete profiles MUST deny as ambiguous or invalid.

#### Scenario: Exact allow rule passes

- GIVEN a deny-by-default profile has one exact allow rule for the normalized request
- WHEN the decision core evaluates the request
- THEN it returns allow with the selected rule identity

#### Scenario: Conflicting rules deny

- GIVEN equal-priority matching rules produce conflicting decisions
- WHEN the decision core evaluates the request
- THEN it returns an ambiguity denial and no allow capability

### Requirement: Contract, UCAN, and approval boundaries remain separate

r[contract_backends.runtime_effect_permissions.authority] A runtime effect allow MUST require the configured Basalt contract decision and explicit verified UCAN facts when the profile requires them. Human approval MAY add a consumer gate, but it MUST NOT override a Basalt denial or missing authority.

#### Scenario: Approval cannot widen denial

- GIVEN a client or operator approves an effect that Basalt denies
- WHEN the consumer combines the decisions
- THEN the final admission remains denied

### Requirement: Consumer mappings retain native enforcement

r[contract_backends.runtime_effect_permissions.consumer] Consumers MUST own native request parsing, path admission, endpoint parsing, executable identity, environment access, binding implementation, currentness checks, and effect execution. Filesystem prefix policy MUST NOT replace Cap Root or capability-relative I/O.

#### Scenario: Lossy mapping is rejected

- GIVEN a consumer cannot map a native effect request to the portable subject without losing behavior-relevant facts
- WHEN mapping validation runs
- THEN the consumer denies before calling the permission core

### Requirement: Permission receipts are metadata-only

r[contract_backends.runtime_effect_permissions.evidence] Basalt MUST emit deterministic BLAKE3-bound permission receipts with policy, request, selected rule, authority input, decision, diagnostic, and non-claim identities. Receipts MUST exclude credentials, environment values, bearer tokens, raw host paths, and effect payloads.

#### Scenario: Receipt does not prove enforcement

- GIVEN a passing permission receipt exists
- WHEN a consumer prepares an effect
- THEN the consumer still rechecks current policy, capability, adapter, and native subject facts

### Requirement: Permission policy has positive and negative verification

r[contract_backends.runtime_effect_permissions.verification] The focused rail MUST cover all supported scopes and negative unknown, malformed, ambiguous, stale, missing-authority, lossy-mapping, and overclaim cases.

#### Scenario: Focused rail passes

- GIVEN policy freshness, Rust parity, decision fixtures, receipts, and consumer mappings are current
- WHEN the focused rail runs
- THEN it reports the supported decision boundary and non-claims
