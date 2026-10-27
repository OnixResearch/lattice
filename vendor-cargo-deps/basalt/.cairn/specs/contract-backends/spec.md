# Contract Backends Specification

## Purpose

Defines the `contract-backends` capability for UCAN contract language backends, including Nickel compatibility, Steel Scheme support, and higher-order callable contract descriptors.

## Requirements

### Requirement: The crate MUST expose a backend-neutral UCAN contract envelope for Nickel and Steel contracts.

r[contract_backends.steel_contract_backend.shared_envelope] The crate MUST expose a backend-neutral UCAN contract envelope that identifies the contract backend, contract id/version, normalized source hash, input schema, output schema, and receipt schema version.

#### Scenario: Nickel and Steel backend identity

r[contract_backends.steel_contract_backend.shared_envelope.backend_identity]

- GIVEN a consumer loads a UCAN contract envelope
- WHEN the envelope names a supported `nickel` or `steel` backend
- THEN Rust validation preserves the backend identity and source binding before evaluating or trusting backend-specific evidence

#### Scenario: Unsupported backend denial

r[contract_backends.steel_contract_backend.shared_envelope.unsupported_backend]

- GIVEN a consumer loads a UCAN contract envelope with an unsupported backend id
- WHEN Rust validation runs
- THEN the crate returns a deterministic denied or error receipt without attempting backend evaluation

### Requirement: Steel contract evaluation MUST use bounded DTO inputs and structured receipts only.

r[contract_backends.steel_contract_backend.steel_dto_boundary] Steel contract evaluation MUST use bounded DTO inputs and structured receipts only, with Rust retaining authority enforcement and receipt validation.

#### Scenario: Steel predicate request

r[contract_backends.steel_contract_backend.steel_dto_boundary.predicate_request]

- GIVEN a Steel-backed contract is selected for a UCAN caveat or capability predicate
- WHEN the evaluation request is built
- THEN the request contains only bounded contract DTO fields, declared input data, source/hash metadata, and declared capability context

#### Scenario: No ambient Steel authority

r[contract_backends.steel_contract_backend.steel_dto_boundary.no_ambient_authority]

- GIVEN Steel contract code is evaluated through the crate's backend adapter
- WHEN the code requests filesystem, process, network, credential, mutation, or undeclared host capability access
- THEN the adapter denies the request or omits the host function so Rust emits a deterministic denied/error receipt

### Requirement: Higher-order contract descriptors MUST make callable/predicate values explicit and hash-bound.

r[contract_backends.steel_contract_backend.higher_order_descriptors] Higher-order contract descriptors MUST make callable or predicate values explicit with stable callable id, arity, argument contracts, return contract, required UCAN capabilities, determinism/effect class, source/content hash, and redaction class.

#### Scenario: Callable descriptor accepted

r[contract_backends.steel_contract_backend.higher_order_descriptors.callable_descriptor]

- GIVEN a Steel contract exports a predicate or callable used by a UCAN caveat
- WHEN Rust validates the higher-order descriptor
- THEN arity, argument contracts, return contract, capability requirements, determinism/effect class, source hash, and redaction class are present before invocation is allowed

#### Scenario: Opaque callable rejected

r[contract_backends.steel_contract_backend.higher_order_descriptors.opaque_callable_rejected]

- GIVEN a Steel contract attempts to pass an opaque closure or unhashable callable across the UCAN boundary
- WHEN Rust validates the higher-order descriptor
- THEN validation fails closed with a deterministic issue code and without invoking the callable

### Requirement: Steel backend receipts MUST be deterministic and authority-bound.

r[contract_backends.steel_contract_backend.receipts] Steel backend receipts MUST bind backend id, contract id/version, normalized source hash, request hash, decision, caveat evidence, declared effect class, and required capability checks.

#### Scenario: Receipt hash stability

r[contract_backends.steel_contract_backend.receipts.hash_stability]

- GIVEN the same Steel contract request is evaluated with the same source, policy, capabilities, and input DTO
- WHEN receipt hashing runs
- THEN the receipt hash is stable across repeated runs and excludes display-only or secret material

#### Scenario: Missing authority evidence rejected

r[contract_backends.steel_contract_backend.receipts.missing_authority_evidence]

- GIVEN a Steel backend receipt claims an allow decision for an effectful or capability-bearing callable
- WHEN required UCAN capability evidence is absent or mismatched
- THEN Rust validation rejects the receipt and returns a deterministic denied/error result

### Requirement: Nickel compatibility MUST remain intact when Steel support is added.

r[contract_backends.steel_contract_backend.nickel_compatibility] Nickel compatibility MUST remain intact when Steel support is added.

#### Scenario: Existing Nickel policies still validate

r[contract_backends.steel_contract_backend.nickel_compatibility.existing_nickel_policies]

- GIVEN an existing Nickel-authored policy artifact is consumed by the crate
- WHEN Steel backend support is present but not selected
- THEN existing Nickel validation, generated artifact freshness, and allow/deny receipt behavior are preserved

### Requirement: Basalt MUST prove the Steel DTO boundary against a runtime-adapter-shaped fixture.

r[contract_backends.steel_runtime_adapter_fixture.adapter_fixture] Basalt MUST prove the Steel DTO boundary against a runtime-adapter-shaped fixture while keeping Rust responsible for authority and receipt validation.

#### Scenario: Adapter-shaped evaluation

r[contract_backends.steel_runtime_adapter_fixture.adapter_fixture.evaluation]

- GIVEN a Steel-backed contract request is prepared by Rust
- WHEN an adapter-shaped fixture evaluates the request and returns a structured receipt
- THEN Rust validates the request hash, source hash, decision, evidence, and capability binding before accepting the result

### Requirement: The Steel adapter fixture MUST deny or omit ambient host authority.

r[contract_backends.steel_runtime_adapter_fixture.no_ambient_authority] The Steel adapter fixture MUST deny or omit filesystem, process, network, credential, mutation, and undeclared host capability authority.

#### Scenario: Host authority blocked

r[contract_backends.steel_runtime_adapter_fixture.no_ambient_authority.blocked_host_action]

- GIVEN Steel code or a fixture requests an undeclared host action
- WHEN the adapter boundary processes the request
- THEN the result is a deterministic denied/error receipt and no host side effect is performed

### Requirement: Runtime-adapter receipts MUST fail closed on malformed or drifted evidence.

r[contract_backends.steel_runtime_adapter_fixture.receipt_validation] Runtime-adapter receipts MUST fail closed when request hashes, source hashes, capability evidence, effect class, or output schemas do not match the validated request.

#### Scenario: Drifted receipt rejected

r[contract_backends.steel_runtime_adapter_fixture.receipt_validation.drift_rejected]

- GIVEN an adapter returns a receipt with mismatched request/source hashes, missing authority evidence, or malformed output
- WHEN Rust validates the receipt
- THEN Basalt rejects it deterministically without treating the adapter decision as authoritative

### Requirement: Policy-boundary audit profiles

r[basalt.ast_grep_policy_audits.profile]

Basalt MUST define ast-grep audit profiles for syntax-level policy-boundary checks with declared scan scope, release posture, severity, and non-claims.

#### Scenario: Policy audit is advisory

- GIVEN a Basalt ast-grep policy-boundary rule is configured as advisory
- WHEN the release verifier runs the scan
- THEN findings MUST be reported as candidate boundary evidence and MUST NOT block release by default.

### Requirement: Receipt identity

r[basalt.ast_grep_policy_audits.identity]

Basalt MUST bind ast-grep tool identity, rule bundle BLAKE3 identity, scan scope, release verifier run identity, findings summary, and non-claims into audit receipts.

#### Scenario: Receipt references stale rule bundle

- GIVEN a release receipt references an ast-grep rule bundle identity
- WHEN the rule bundle changes before review
- THEN Basalt MUST require a fresh scan receipt before treating the audit as current.

### Requirement: Positive and negative fixtures

r[basalt.ast_grep_policy_audits.fixtures]

Basalt MUST require positive and negative fixtures before an ast-grep policy-boundary rule can become release-blocking.

#### Scenario: Blocking rule lacks negative fixture

- GIVEN a policy-boundary rule is proposed for blocking posture without a negative fixture
- WHEN Basalt validates the rule pack
- THEN validation MUST fail closed and keep the rule advisory.

### Requirement: Release receipt reporting

r[basalt.ast_grep_policy_audits.release_receipts]

Basalt SHOULD report ast-grep audit outcomes in release verification receipts as structural candidate evidence with remediation guidance and explicit non-claims.

#### Scenario: Enforcement bypass candidate is reported

- GIVEN ast-grep reports a candidate direct bypass around the enforcement helper
- WHEN the release receipt is emitted
- THEN the receipt SHOULD identify the rule, span, severity, and remediation note while preserving that authorization was not proven or disproven.

### Requirement: Shell-owned scan execution

r[basalt.ast_grep_policy_audits.shell_boundary]

Basalt MUST keep ast-grep process execution, filesystem scans, and raw output capture in release verifier or CLI shell code.

#### Scenario: Core enforcement runs

- GIVEN core enforcement evaluates a UCAN-backed request
- WHEN policy validation executes
- THEN it MUST NOT depend on ast-grep state, filesystem scans, process execution, or ambient scan results.

### Requirement: Audit validation evidence

r[basalt.ast_grep_policy_audits.validation]

Basalt MUST validate ast-grep policy audits with rule tests, fixture scans, release verifier receipt checks, Cairn gates, and focused release rails.

#### Scenario: Audit change is reviewed

- GIVEN ast-grep policy-boundary audit behavior changes
- WHEN validation evidence is assembled
- THEN evidence MUST include positive fixtures, negative fixtures, release receipt output, lifecycle gates, and focused Basalt verifier results.

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

## World-branch authority requirements

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
