# Repository Validation Specification

## Purpose

Defines the `repository-validation` capability.

## Requirements

### Requirement: Host-secret-independent repository checks
r[n8nclone.repository_validation.flake_check] The repository MUST provide a full Nix flake validation path that does not require a machine signing secret and MUST NOT weaken signing for ordinary package, cache, or release builds.

#### Scenario: Missing host signing key
r[n8nclone.repository_validation.flake_check.scenario.missing-signing-key]
- GIVEN the host configures a runtime `secret-key-files` path whose key is unavailable
- WHEN an operator runs the documented check-scoped flake validation command
- THEN validation MUST proceed with signing disabled only for that invocation and MUST NOT require a placeholder key

#### Scenario: Normal signing policy remains external
r[n8nclone.repository_validation.flake_check.scenario.normal-signing]
- GIVEN an operator runs an ordinary Nix build without the check-scoped override
- WHEN the host has configured signing policy
- THEN the repository MUST leave that policy unchanged and MUST NOT embed a flake-level signing override

### Requirement: Reproducible vendored source snapshot
r[n8nclone.repository_validation.vendor_snapshot] The Git and Nix source snapshots MUST include every Rust source file required by configured vendored path dependencies.

#### Scenario: Nested source resembles an output name
r[n8nclone.repository_validation.vendor_snapshot.scenario.nested-output-name]
- GIVEN a vendored Rust source path contains a basename or directory named like a repository output such as `result` or `coverage`
- WHEN Git ignore admission and Nix source snapshotting run
- THEN the source file MUST remain admitted and available to Cargo

### Requirement: Complete Nix test source
r[n8nclone.repository_validation.test_source] The Nix source closure MUST include tracked non-Rust fixtures required by repository-owned test targets.

#### Scenario: Integration test embeds a JSON fixture
r[n8nclone.repository_validation.test_source.scenario.json-fixture]
- GIVEN a repository-owned Rust test includes a tracked JSON fixture
- WHEN the Nix Clippy or Nextest check compiles that target
- THEN the fixture MUST be present in the filtered source closure

### Requirement: Formatter ownership boundary
r[n8nclone.repository_validation.format_scope] The flake format check MUST validate repository-owned Rust packages without requiring imported vendored snapshots to conform to the consumer toolchain's formatter output.

#### Scenario: Vendored source uses upstream formatting
r[n8nclone.repository_validation.format_scope.scenario.vendor-format]
- GIVEN a vendored path dependency uses formatting that differs from the current project toolchain
- WHEN the flake format check runs
- THEN the check MUST format-check `lattice` and MUST NOT fail solely because vendored source would be reformatted

### Requirement: Clippy ownership boundary
r[n8nclone.repository_validation.clippy_scope] The flake Clippy check MUST deny warnings for repository-owned targets without applying repository lint policy to vendored dependencies.

#### Scenario: Vendored dependency has upstream lint findings
r[n8nclone.repository_validation.clippy_scope.scenario.vendor-lints]
- GIVEN a vendored path dependency triggers Clippy lints under the consumer's warning policy
- WHEN the flake Clippy check runs
- THEN the check MUST lint `lattice` with dependencies excluded from lint admission

### Requirement: Repository-owned traceability sources
r[lattice.repository_validation.traceability_sources] Lattice MUST evaluate accepted requirement linkage against bounded repository-owned source roots and MUST preserve fail-closed missing and dangling diagnostics.

#### Scenario: Lattice markers are discovered
r[lattice.repository_validation.traceability_sources.scenario.discovery]
- GIVEN accepted requirements live under `cairn/specs` and Lattice-owned markers live under the configured source, test, example, script, and policy roots
- WHEN an operator runs `cairn tracey coverage --root .` with the checked repository policy
- THEN the command MUST discover those markers and MUST NOT fall back to unrelated `crates` or `tools` roots

#### Scenario: Missing and dangling linkage remains fail closed
r[lattice.repository_validation.traceability_sources.scenario.fail-closed]
- GIVEN correct source discovery reveals accepted requirements without markers or markers without accepted requirements
- WHEN Tracey computes coverage
- THEN it MUST report the missing and dangling identifiers, render `valid: false`, and exit unsuccessfully

#### Scenario: Unsafe source roots are rejected
r[lattice.repository_validation.traceability_sources.scenario.unsafe-root]
- GIVEN a policy fixture configures an absolute, parent-traversing, or otherwise unsafe traceability root
- WHEN Cairn validates the Nickel policy export
- THEN export MUST fail before the root can contribute requirement evidence

#### Scenario: Stale marker identifiers are reconciled without false coverage
r[lattice.repository_validation.traceability_sources.scenario.marker-reconciliation]
- GIVEN a discovered marker names no accepted requirement
- WHEN the marker is reconciled against its observed implementation or test behavior
- THEN Lattice MUST add an exact accepted requirement, bind only to an already accepted requirement the evidence supports, or remove the unsupported marker while preserving unrelated missing findings

### Requirement: Maintained traceability debt inventory
r[lattice.repository_validation.traceability_debt_inventory] Lattice MUST derive a deterministic, BLAKE3-bound inventory from the exact Cairn missing-requirement set and an explicitly reviewed typed policy without promoting inventory rows into requirement evidence.

#### Scenario: Complete deterministic partition
r[lattice.repository_validation.traceability_debt_inventory.scenario.complete]
- GIVEN a Cairn coverage receipt with a policy-pinned missing set and no dangling markers
- WHEN the operator generates or checks the debt inventory
- THEN every missing identifier MUST appear exactly once in sorted subsystem and review-state summaries and repeated generation MUST produce the same inventory identity

#### Scenario: Stale or malformed inputs fail closed
r[lattice.repository_validation.traceability_debt_inventory.scenario.fail_closed]
- GIVEN duplicate missing identifiers, a stale reviewed identifier, an unsafe evidence path, a changed missing-set identity, or a tampered report
- WHEN the inventory core validates or checks the inputs
- THEN it MUST emit bounded deterministic issue codes and MUST NOT report a valid inventory

#### Scenario: Unreviewed debt remains an explicit non-claim
r[lattice.repository_validation.traceability_debt_inventory.scenario.non_claim]
- GIVEN a missing identifier has no explicit reviewed policy decision
- WHEN the inventory is generated
- THEN the row MUST remain `unassessed` and MUST NOT claim evidence absence, requirement satisfaction, semantic correctness, complete traceability, or release readiness

### Requirement: Reviewed Tracey evidence promotion
r[lattice.repository_validation.reviewed_tracey_promotion] Lattice MUST promote reviewed traceability candidates only through markers attached to exact maintained assertions and MUST preserve every unrelated missing or dangling linkage state.

#### Scenario: Exact allowlisted missing-set delta
r[lattice.repository_validation.reviewed_tracey_promotion.scenario.exact_delta]
- GIVEN a baseline failing coverage receipt and an explicit reviewed promotion allowlist
- WHEN the maintained promotion check compares the current failing coverage receipt
- THEN the removed identifiers MUST equal the allowlist, no identifier MUST be added, dangling MUST remain empty, and every unrelated missing identifier MUST remain unchanged

#### Scenario: Markers remain locally bound
r[lattice.repository_validation.reviewed_tracey_promotion.scenario.local_binding]
- GIVEN a reviewed candidate has positive or negative assertions needed for its complete accepted scenario
- WHEN the candidate is promoted
- THEN every marker MUST be attached to an exact supporting test and MUST NOT rely on file-level presence, lexical similarity, generated inventory rows, or archived evidence

#### Scenario: Unexpected promotion fails closed
r[lattice.repository_validation.reviewed_tracey_promotion.scenario.fail_closed]
- GIVEN a duplicate baseline identifier, an unexpected removed identifier, an added identifier, a dangling marker, or successful coverage
- WHEN the maintained promotion check validates the transition
- THEN it MUST return deterministic issue codes and MUST NOT accept the promotion

### Requirement: Bounded traceability tranche review
r[lattice.repository_validation.traceability_tranche_review] Lattice MUST review a selected missing-requirement tranche through explicit typed classifications and maintained evidence identities without changing requirement coverage.

#### Scenario: Complete explicit tranche partition
r[lattice.repository_validation.traceability_tranche_review.scenario.complete]
- GIVEN a bounded subsystem tranche and an explicit expectation table
- WHEN the maintained review is validated
- THEN every selected missing identifier MUST have exactly one reviewed non-`unassessed` classification and no unrelated identifier MUST be classified by that tranche

#### Scenario: Evidence strength remains explicit
r[lattice.repository_validation.traceability_tranche_review.scenario.evidence]
- GIVEN exact, partial, and no-exact review decisions
- WHEN evidence bindings are checked
- THEN exact and partial rows MUST bind current safe file-byte identities, no-exact rows MUST remain evidence-free, and the review MUST NOT satisfy or promote any selected requirement

#### Scenario: Classification or coverage drift fails closed
r[lattice.repository_validation.traceability_tranche_review.scenario.fail_closed]
- GIVEN a missing expected row, unexpected reviewed row, classification drift, rationale drift, evidence-path drift, missing-set change, dangling marker, or successful coverage
- WHEN the maintained review validator checks the tranche
- THEN it MUST return deterministic issue codes and MUST NOT accept the review

### Requirement: Core credential traceability audit
r[lattice.repository_validation.credential_traceability_audit] Lattice MUST review the selected core credential traceability tranche through explicit typed classifications and current maintained evidence identities without changing credential behavior or satisfying reviewed credential requirements.

#### Scenario: Complete bounded credential partition
r[lattice.repository_validation.credential_traceability_audit.scenario.complete]
- GIVEN the selected 12 missing core credential identifiers and an explicit expectation table
- WHEN the maintained audit is validated
- THEN every selected identifier MUST have exactly one reviewed non-`unassessed` classification and no unselected credential identifier MUST be classified by this audit

#### Scenario: Credential evidence strength remains explicit
r[lattice.repository_validation.credential_traceability_audit.scenario.evidence]
- GIVEN exact and partial credential evidence with different assertion strength
- WHEN evidence bindings are checked
- THEN every reviewed row MUST bind current safe file-byte identities, partial rows MUST remain explicitly partial, and no reviewed credential marker MUST be added

#### Scenario: Credential audit drift fails closed
r[lattice.repository_validation.credential_traceability_audit.scenario.fail_closed]
- GIVEN a missing or unexpected row, classification drift, rationale drift, evidence-path drift, missing-set change, dangling marker, or successful coverage
- WHEN the maintained credential audit validator checks the tranche
- THEN it MUST return deterministic issue codes and MUST NOT accept the audit

### Requirement: Core credential evidence promotion
r[lattice.repository_validation.core_credential_promotion] Lattice MUST promote the reviewed core credential tranche only through markers attached to exact maintained tests and MUST preserve every unrelated missing or dangling linkage state.

#### Scenario: Exact 12-ID credential delta
r[lattice.repository_validation.core_credential_promotion.scenario.exact_delta]
- GIVEN the failing 353-item baseline receipt and the explicit 12-ID credential allowlist
- WHEN the maintained promotion check compares the current failing receipt
- THEN baseline minus current MUST equal the allowlist, current minus baseline MUST be empty, and missing coverage MUST equal 341

#### Scenario: Credential markers remain locally bound
r[lattice.repository_validation.core_credential_promotion.scenario.local_binding]
- GIVEN each reviewed credential scenario has exact maintained assertions
- WHEN its marker is promoted
- THEN the marker MUST be attached directly to an exact supporting test and MUST NOT rely on parent markers, file scope, lexical similarity, generated inventory rows, or archived evidence

#### Scenario: Credential promotion drift fails closed
r[lattice.repository_validation.core_credential_promotion.scenario.fail_closed]
- GIVEN a duplicate allowlist or receipt ID, missing expected removal, unrelated removal, added ID, dangling marker, stale reviewed policy row, or successful coverage
- WHEN the maintained promotion validator checks the transition
- THEN it MUST return deterministic issue codes and MUST NOT accept the promotion

### Requirement: Remaining credential traceability audit
r[lattice.repository_validation.remaining_credential_audit] Lattice MUST maintain a bounded review of exactly the remaining 14 credential debt rows without changing credential behavior or promoting reviewed identifiers.

#### Scenario: Complete remaining credential tranche
r[lattice.repository_validation.remaining_credential_audit.scenario.complete]
- GIVEN the 341-item missing-set baseline after the core credential promotion
- WHEN the remaining credential audit runs
- THEN it MUST classify exactly the 14 allowlisted credential rows once each and MUST leave every other row unassessed

#### Scenario: Evidence strength remains byte-bound
r[lattice.repository_validation.remaining_credential_audit.scenario.evidence]
- GIVEN a selected credential row has exact, partial, or no-exact maintained evidence
- WHEN the audit records its decision
- THEN the classification, bounded rationale, safe evidence paths, and current file-byte BLAKE3 hashes MUST match the reviewed expectation without treating parent markers, archives, inventory rows, or implementation shape as stronger evidence

#### Scenario: Remaining credential audit fails closed
r[lattice.repository_validation.remaining_credential_audit.scenario.fail_closed]
- GIVEN a selected row is missing, duplicated, unexpected, reordered, reclassified, rebound, promoted, or accompanied by missing-set or dangling-marker drift
- WHEN maintained audit validation runs
- THEN it MUST reject the audit with deterministic issue codes while global coverage remains unsuccessful

### Requirement: Webhook cookbook credential evidence promotion
r[lattice.repository_validation.cookbook_credential_promotion] Lattice MUST promote only the reviewed webhook cookbook credential row through markers attached to its exact maintained tests and MUST preserve every unrelated missing or dangling linkage state.

#### Scenario: Exact one-ID cookbook delta
r[lattice.repository_validation.cookbook_credential_promotion.scenario.exact_delta]
- GIVEN the failing 341-item baseline receipt and the one-ID cookbook allowlist
- WHEN the maintained promotion check compares the current failing receipt
- THEN baseline minus current MUST equal the allowlist, current minus baseline MUST be empty, and missing coverage MUST equal 340

#### Scenario: Cookbook marker remains locally bound
r[lattice.repository_validation.cookbook_credential_promotion.scenario.local_binding]
- GIVEN the reviewed cookbook row has exact maintained accepted and rejected smoke assertions
- WHEN its marker is promoted
- THEN the marker MUST attach directly to both exact supporting tests and MUST NOT rely on child markers, file scope, lexical similarity, generated inventory rows, or archived evidence

#### Scenario: Cookbook promotion drift fails closed
r[lattice.repository_validation.cookbook_credential_promotion.scenario.fail_closed]
- GIVEN a duplicate allowlist or receipt ID, missing expected removal, unrelated removal, added ID, dangling marker, stale reviewed policy row, or successful coverage
- WHEN the maintained promotion validator checks the transition
- THEN it MUST return deterministic issue codes and MUST NOT accept the promotion

### Requirement: n8n credential boundary evidence completion
r[lattice.repository_validation.n8n_credential_completion] Lattice MUST maintain exact byte-bound evidence for the four reviewed n8n credential rows without promoting their requirement markers or changing unrelated credential classifications.

#### Scenario: Import and export boundaries are observed
r[lattice.repository_validation.n8n_credential_completion.scenario.boundaries]
- GIVEN opaque and plaintext n8n credential imports plus a provider-resolved reference-bearing workflow
- WHEN import execution, rejection, save, raw-store inspection, and CLI export run
- THEN exact assertions MUST observe provider resolution, pre-side-effect failure, store non-creation, opaque exported identity, and plaintext omission

#### Scenario: Parity surfaces preserve the bounded exception
r[lattice.repository_validation.n8n_credential_completion.scenario.parity]
- GIVEN the designated plaintext rejection fixture and every other maintained parity fixture and report
- WHEN import reports, diagnostics, and CLI output are inspected
- THEN only the designated input fixture MAY contain exactly one sentinel and every output surface MUST omit it while retaining reference guidance

#### Scenario: Exactly four rows become exact
r[lattice.repository_validation.n8n_credential_completion.scenario.exact]
- GIVEN the reviewed 340-item inventory and the four-ID n8n credential tranche
- WHEN the completion policy is generated
- THEN exactly those four rows MUST be exact, the other reviewed rows MUST remain unchanged, and none of the four IDs may gain an implementation or verification marker

#### Scenario: n8n completion drift fails closed
r[lattice.repository_validation.n8n_credential_completion.scenario.fail_closed]
- GIVEN a row omission, duplicate, reclassification, rationale or evidence drift, selected marker, missing-set change, dangling marker, or successful coverage
- WHEN the maintained completion validator checks the tranche
- THEN it MUST reject the drift with deterministic issue codes

### Requirement: n8n credential boundary evidence promotion
r[lattice.repository_validation.n8n_credential_promotion] Lattice MUST promote only the four reviewed exact n8n credential boundary rows through markers attached to their maintained tests and MUST preserve every unrelated missing or dangling linkage state.

#### Scenario: Exact four-ID n8n credential delta
r[lattice.repository_validation.n8n_credential_promotion.scenario.exact_delta]
- GIVEN the failing 340-item baseline receipt and the explicit four-ID n8n credential allowlist
- WHEN the maintained promotion check compares the current failing receipt
- THEN baseline minus current MUST equal the allowlist, current minus baseline MUST be empty, and missing coverage MUST equal 336

#### Scenario: n8n credential markers remain locally bound
r[lattice.repository_validation.n8n_credential_promotion.scenario.local_binding]
- GIVEN each reviewed n8n credential row has exact maintained boundary assertions
- WHEN its marker is promoted
- THEN each marker MUST attach directly to every required supporting test and MUST NOT rely on implementation names, file scope, lifecycle classification tests, generated inventory rows, or archived evidence

#### Scenario: n8n credential promotion drift fails closed
r[lattice.repository_validation.n8n_credential_promotion.scenario.fail_closed]
- GIVEN a duplicate allowlist or receipt ID, missing expected removal, unrelated removal, added ID, dangling marker, missing local binding, stale reviewed policy row, or successful coverage
- WHEN the maintained promotion validator checks the transition
- THEN it MUST return deterministic issue codes and MUST NOT accept the promotion

### Requirement: Provider credential boundary evidence completion
r[lattice.repository_validation.provider_credential_completion] Lattice MUST complete only provider credential rows whose accepted boundaries are directly asserted and BLAKE3-bound, MUST add no selected provider markers, and MUST preserve every missing and dangling linkage state.

#### Scenario: Provider diagnostics are safe before rendering
r[lattice.repository_validation.provider_credential_completion.scenario.diagnostics]
- GIVEN a provider succeeds or returns arbitrary backend text derived from secret material
- WHEN the probe result is constructed and rendered
- THEN output MUST retain only opaque reference identity, reachability, and fixed safe status text and MUST omit backend and secret text

#### Scenario: Provider boundaries are directly observed
r[lattice.repository_validation.provider_credential_completion.scenario.boundaries]
- GIVEN cache, command, local-vault, timeout, tamper, workflow-file, and executor boundaries
- WHEN maintained positive and negative integration tests run
- THEN every reviewed provider clause claimed exact MUST have a direct assertion at its accepted boundary

#### Scenario: Exact provider classifications remain bounded
r[lattice.repository_validation.provider_credential_completion.scenario.exact]
- GIVEN the reviewed provider rows and current maintained evidence bytes
- WHEN completion validation runs
- THEN only exact rows MUST be reclassified, evidence paths MUST bind current BLAKE3 hashes, selected markers MUST remain absent, and the 336-item missing set MUST remain unchanged

#### Scenario: Provider completion drift fails closed
r[lattice.repository_validation.provider_credential_completion.scenario.fail_closed]
- GIVEN classification, rationale, evidence, marker, missing-set, dangling, or successful-coverage drift
- WHEN the maintained completion validator runs
- THEN it MUST return deterministic issue codes and MUST NOT accept completion

### Requirement: Provider credential evidence promotion
r[lattice.repository_validation.provider_credential_promotion] Lattice MUST promote only the seven reviewed exact provider credential rows through markers attached to maintained boundary tests and MUST preserve every unrelated missing or dangling linkage state.

#### Scenario: Exact seven-ID provider delta
r[lattice.repository_validation.provider_credential_promotion.scenario.exact_delta]
- GIVEN the failing 336-item baseline receipt and explicit seven-ID provider allowlist
- WHEN maintained promotion validation compares current failing coverage
- THEN baseline minus current MUST equal the allowlist, current minus baseline MUST be empty, and missing coverage MUST equal 329

#### Scenario: Provider markers remain locally bound
r[lattice.repository_validation.provider_credential_promotion.scenario.local_binding]
- GIVEN every selected provider row has exact maintained boundary assertions
- WHEN its marker is promoted
- THEN each marker MUST attach directly to its required test and MUST NOT rely on implementation names, file scope, lifecycle tests, generated inventory, or archived evidence

#### Scenario: Provider promotion drift fails closed
r[lattice.repository_validation.provider_credential_promotion.scenario.fail_closed]
- GIVEN duplicate, unsorted, missing-removal, unrelated-removal, addition, dangling, successful-coverage, missing-binding, or stale-policy drift
- WHEN the maintained promotion validator runs
- THEN it MUST return deterministic issue codes and MUST NOT accept promotion

### Requirement: Webhook authentication evidence completion
r[lattice.repository_validation.webhook_auth_completion] Lattice MUST complete only webhook authentication rows whose valid and rejected durable boundaries are directly asserted and BLAKE3-bound, MUST add no selected markers, and MUST preserve unrelated missing and dangling state.

#### Scenario: Safe credential-reference context
r[lattice.repository_validation.webhook_auth_completion.scenario.context]
- GIVEN authentication resolves an opaque credential reference
- WHEN rejection diagnostics and history are rendered or persisted
- THEN they MUST include trigger identity and a domain-separated BLAKE3 reference fingerprint while omitting raw provider, key, token, and secret values

#### Scenario: Webhook persistence boundaries are observed
r[lattice.repository_validation.webhook_auth_completion.scenario.boundaries]
- GIVEN valid and invalid authenticated webhook requests
- WHEN admission, execution, response rendering, history persistence, and raw-store inspection run
- THEN maintained assertions MUST prove valid admission and complete plaintext omission from every inspected durable surface

#### Scenario: Exact webhook classifications remain bounded
r[lattice.repository_validation.webhook_auth_completion.scenario.exact]
- GIVEN the two reviewed webhook rows and current evidence bytes
- WHEN completion validation runs
- THEN only exact rows MUST be reclassified, evidence MUST bind current BLAKE3 hashes, selected markers MUST remain absent, and the 329-item missing set MUST remain unchanged

#### Scenario: Webhook completion drift fails closed
r[lattice.repository_validation.webhook_auth_completion.scenario.fail_closed]
- GIVEN classification, rationale, evidence, marker, missing-set, dangling, or successful-coverage drift
- WHEN maintained completion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept completion

### Requirement: Webhook authentication evidence promotion
r[lattice.repository_validation.webhook_auth_promotion] Lattice MUST promote only the two reviewed exact webhook authentication rows through markers attached to maintained durable-boundary tests and MUST preserve every unrelated missing or dangling linkage state.

#### Scenario: Exact two-ID webhook delta
r[lattice.repository_validation.webhook_auth_promotion.scenario.exact_delta]
- GIVEN the failing 329-item baseline and explicit two-ID webhook allowlist
- WHEN maintained promotion validation compares current coverage
- THEN baseline minus current MUST equal the allowlist, current minus baseline MUST be empty, and missing coverage MUST equal 327

#### Scenario: Webhook markers remain locally bound
r[lattice.repository_validation.webhook_auth_promotion.scenario.local_binding]
- GIVEN both rows have exact maintained durable-boundary assertions
- WHEN their markers are promoted
- THEN each marker MUST attach directly to its required test and MUST NOT rely on implementation names, file scope, lifecycle tests, generated inventory, or archived evidence

#### Scenario: Webhook promotion drift fails closed
r[lattice.repository_validation.webhook_auth_promotion.scenario.fail_closed]
- GIVEN duplicate, unsorted, missing-removal, unrelated-removal, addition, dangling, successful-coverage, missing-binding, or stale-policy drift
- WHEN maintained promotion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept promotion

### Requirement: Workflow-example traceability audit
r[lattice.repository_validation.workflow_example_audit] Lattice MUST maintain a bounded explicit review of both missing workflow-example scenarios without changing behavior or promoting reviewed identifiers.

#### Scenario: Complete workflow-example partition
r[lattice.repository_validation.workflow_example_audit.scenario.complete]
- GIVEN the two missing workflow-example scenario identifiers and an explicit expectation table
- WHEN maintained audit validation runs
- THEN each identifier MUST have exactly one non-`unassessed` classification and no unrelated row MUST be classified by this audit

#### Scenario: Workflow-example evidence strength remains explicit
r[lattice.repository_validation.workflow_example_audit.scenario.evidence]
- GIVEN direct end-to-end evidence for the selfie example and no maintained invalid-example fixture
- WHEN evidence bindings are checked
- THEN the selfie row MUST bind current safe file-byte identities, the invalid row MUST remain evidence-free, no selected marker may be added, and coverage MUST remain unchanged at 327 missing IDs

#### Scenario: Workflow-example audit drift fails closed
r[lattice.repository_validation.workflow_example_audit.scenario.fail_closed]
- GIVEN a missing or unexpected row, classification or rationale drift, unsafe or stale evidence, a selected marker, missing-set drift, a dangling marker, or successful coverage
- WHEN maintained audit validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept the audit

### Requirement: Node-template prompt traceability audit
r[lattice.repository_validation.node_template_prompt_audit] Lattice MUST maintain a bounded explicit review of all three missing node-template prompt rows without changing behavior or promoting reviewed identifiers.

#### Scenario: Complete node-template prompt partition
r[lattice.repository_validation.node_template_prompt_audit.scenario.complete]
- GIVEN the three missing node-template prompt identifiers and an explicit expectation table
- WHEN maintained audit validation runs
- THEN each identifier MUST have exactly one non-`unassessed` classification and no unrelated row MUST be classified by this audit

#### Scenario: Node-template prompt evidence strength remains explicit
r[lattice.repository_validation.node_template_prompt_audit.scenario.evidence]
- GIVEN exact collection and cancellation assertions plus unasserted user-visible required-prompt rendering
- WHEN evidence bindings are checked
- THEN exact and partial rows MUST bind current safe file-byte identities, partial evidence MUST remain partial, no selected marker may be added, and coverage MUST remain unchanged at 327 missing IDs

#### Scenario: Node-template prompt audit drift fails closed
r[lattice.repository_validation.node_template_prompt_audit.scenario.fail_closed]
- GIVEN a missing or unexpected row, classification or rationale drift, unsafe or stale evidence, a selected marker, missing-set drift, a dangling marker, or successful coverage
- WHEN maintained audit validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept the audit

### Requirement: Exact node-template prompt evidence completion
r[lattice.repository_validation.node_template_prompt_completion] Lattice MUST maintain direct assertion-bound evidence for every clause in the three reviewed node-template prompt requirements before any marker promotion.

#### Scenario: Required prompt rendering is directly asserted
r[lattice.repository_validation.node_template_prompt_completion.scenario.rendered]
- GIVEN an add-node flow positioned at a required template parameter before node creation
- WHEN the pure panel-rendering core produces user-visible prompt lines
- THEN maintained assertions MUST inspect the parameter sequence, name, type, required label, description, current value, and controls, while a negative companion MUST reject a required label for an optional parameter

#### Scenario: Complete prompt tranche becomes exact without promotion
r[lattice.repository_validation.node_template_prompt_completion.scenario.complete]
- GIVEN direct maintained collection, rendering, cancellation, and no-partial-state assertions
- WHEN the completion inventory is generated
- THEN exactly the three node-template prompt rows MUST be `exact_maintained_evidence`, bind current safe file-byte identities, and remain absent from source markers

#### Scenario: Completion drift fails closed
r[lattice.repository_validation.node_template_prompt_completion.scenario.fail_closed]
- GIVEN missing or unexpected rows, non-exact classification, rationale drift, unsafe or stale evidence, a selected marker, missing-set drift, dangling markers, or successful coverage
- WHEN maintained completion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept completion

### Requirement: Exact node-template prompt evidence promotion
r[lattice.repository_validation.node_template_prompt_promotion] Lattice MUST promote exactly the three completed node-template prompt requirements through assertion-local markers and an exact missing-set transition.

#### Scenario: Exact three-ID promotion delta
r[lattice.repository_validation.node_template_prompt_promotion.scenario.exact_delta]
- GIVEN the reviewed completion baseline with 327 missing requirements
- WHEN node-template prompt evidence is promoted
- THEN the current result MUST contain 324 missing requirements, remove exactly the three allowlisted prompt IDs, add no ID, retain zero dangling markers, and remain `valid: false`

#### Scenario: Promoted markers remain assertion-local
r[lattice.repository_validation.node_template_prompt_promotion.scenario.local_binding]
- GIVEN the exact collection, required rendering, and cancellation tests
- WHEN promoted marker bindings are validated
- THEN each identifier MUST occur exactly once immediately before its named supporting test and MUST be absent from reviewed debt policy

#### Scenario: Promotion drift fails closed
r[lattice.repository_validation.node_template_prompt_promotion.scenario.fail_closed]
- GIVEN duplicate or unsorted missing sets, duplicate allowlists, missing bindings, stale policy rows, unrelated removals, additions, dangling markers, or successful coverage
- WHEN deterministic promotion validation runs
- THEN it MUST return explicit issue codes and MUST NOT accept the promotion

### Requirement: Scheduling traceability audit
r[lattice.repository_validation.scheduling_audit] Lattice MUST maintain a bounded explicit review of all seven remaining scheduling debt rows without changing behavior or promoting reviewed identifiers.

#### Scenario: Complete scheduling partition
r[lattice.repository_validation.scheduling_audit.scenario.complete]
- GIVEN the seven missing scheduling identifiers and an explicit expectation table
- WHEN maintained audit validation runs
- THEN each identifier MUST have exactly one non-`unassessed` classification and no unrelated row MUST be classified by this audit

#### Scenario: Scheduling evidence strength remains explicit
r[lattice.repository_validation.scheduling_audit.scenario.evidence]
- GIVEN direct admission, persistence, compatibility, cookbook, and listener evidence
- WHEN evidence bindings are checked
- THEN exact rows MUST cover every accepted clause, partial rows MUST name their direct gap, all reviewed evidence MUST bind current safe file bytes, and no selected marker may be added

#### Scenario: Scheduling audit drift fails closed
r[lattice.repository_validation.scheduling_audit.scenario.fail_closed]
- GIVEN a missing or unexpected row, classification or rationale drift, unsafe or stale evidence, a selected marker, missing-set drift, dangling markers, or successful coverage
- WHEN maintained audit validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept the audit

### Requirement: Exact scheduling evidence completion
r[lattice.repository_validation.scheduling_completion] Lattice MUST maintain direct assertion-bound evidence for every clause in the seven reviewed scheduling requirements before promotion.

#### Scenario: Terminal history fields are directly asserted
r[lattice.repository_validation.scheduling_completion.scenario.history]
- GIVEN an admitted trigger whose workflow reaches a terminal state
- WHEN trigger history is loaded
- THEN maintained assertions MUST inspect trigger identity, workflow, run identity, idempotency key, status, and redaction-safe payload summary

#### Scenario: Schedule compatibility compares complete plans
r[lattice.repository_validation.scheduling_completion.scenario.compatibility]
- GIVEN one recurring schedule and its adapted trigger definition
- WHEN both evaluators run at the same due time
- THEN maintained assertions MUST compare the complete due-run plans for equality

#### Scenario: Listener startup is observed through the real process
r[lattice.repository_validation.scheduling_completion.scenario.listener]
- GIVEN an isolated store with a saved workflow and enabled webhook trigger
- WHEN the operator listener process starts on loopback and receives a bounded valid HTTP request
- THEN maintained assertions MUST observe an accepted response, terminal workflow run, and safe trigger history before owned child teardown

#### Scenario: Complete scheduling tranche becomes exact without promotion
r[lattice.repository_validation.scheduling_completion.scenario.exact]
- GIVEN direct maintained evidence for all seven scheduling rows
- WHEN completion inventory validation runs
- THEN every selected row MUST be exact, bind current safe file bytes, remain marker-free, and preserve all 324 missing IDs

### Requirement: Exact scheduling evidence promotion
r[lattice.repository_validation.scheduling_promotion] Lattice MUST promote exactly the seven completed scheduling requirements through assertion-local markers and an exact missing-set transition.

#### Scenario: Exact scheduling delta
r[lattice.repository_validation.scheduling_promotion.scenario.delta]
- GIVEN the completion baseline with 324 missing requirements
- WHEN scheduling evidence is promoted
- THEN current coverage MUST contain 317 missing requirements, remove exactly the seven allowlisted IDs, add no ID, retain zero dangling markers, and remain `valid: false`

#### Scenario: Scheduling markers remain assertion-local
r[lattice.repository_validation.scheduling_promotion.scenario.bindings]
- GIVEN the direct and composite exact scheduling tests
- WHEN bindings are validated
- THEN every promoted identifier MUST attach only to its required supporting tests and MUST be absent from reviewed debt policy

#### Scenario: Scheduling promotion drift fails closed
r[lattice.repository_validation.scheduling_promotion.scenario.fail_closed]
- GIVEN duplicate or unsorted sets, duplicate allowlists, missing bindings, stale policy, unrelated removal, addition, dangling markers, or successful coverage
- WHEN promotion validation runs
- THEN it MUST return explicit issue codes and reject the transition

### Requirement: Exact workflow-example evidence completion
r[lattice.repository_validation.workflow_example_completion] Lattice MUST maintain direct positive and negative fixture evidence for both reviewed image workflow-example scenarios before promotion.

#### Scenario: Invalid image fixture remains intentional
r[lattice.repository_validation.workflow_example_completion.scenario.fixture]
- GIVEN a repository-owned typed Nickel workflow fixture whose image inputs resolve to unsupported GIF bytes
- WHEN the fixture is imported
- THEN its invalid intent and affected image nodes MUST remain explicit and reviewable

#### Scenario: Invalid image execution leaves no partial completion
r[lattice.repository_validation.workflow_example_completion.scenario.negative]
- GIVEN the invalid image fixture and deterministic in-memory image bytes
- WHEN it runs through the executor
- THEN maintained assertions MUST observe unsupported-format diagnostics, skipped downstream execution, and zero completed nodes

### Requirement: Exact workflow-example evidence promotion
r[lattice.repository_validation.workflow_example_promotion] Lattice MUST promote exactly the two completed workflow-example requirements through assertion-local markers and an exact missing-set transition.

#### Scenario: Exact workflow-example delta
r[lattice.repository_validation.workflow_example_promotion.scenario.delta]
- GIVEN 317 missing requirements
- WHEN both workflow-example rows are promoted
- THEN coverage MUST contain 315 missing requirements, remove only those two IDs, add nothing, retain zero dangling markers, and remain `valid: false`

#### Scenario: Workflow-example bindings remain local
r[lattice.repository_validation.workflow_example_promotion.scenario.bindings]
- GIVEN the complete positive and negative image fixture tests
- WHEN marker bindings are checked
- THEN each ID MUST attach directly to its supporting test and be absent from reviewed debt policy

### Requirement: Repository-check traceability audit
r[lattice.repository_validation.repository_checks_audit] Lattice MUST maintain a bounded explicit review of all 11 remaining repository-validation debt rows without changing check behavior or promoting reviewed identifiers.

#### Scenario: Complete repository-check partition
r[lattice.repository_validation.repository_checks_audit.scenario.complete]
- GIVEN the 11 missing repository-validation identifiers and an explicit expectation table
- WHEN maintained audit validation runs
- THEN each identifier MUST have exactly one non-`unassessed` classification and no unrelated row MUST be classified by this audit

#### Scenario: Repository-check evidence strength remains explicit
r[lattice.repository_validation.repository_checks_audit.scenario.evidence]
- GIVEN direct and partial signing, source-closure, formatter, and lint evidence
- WHEN evidence bindings are checked
- THEN exact rows MUST cover every accepted clause, partial rows MUST name their direct observation gap, all reviewed evidence MUST bind current safe file bytes, and no selected marker may be added

#### Scenario: Repository-check audit drift fails closed
r[lattice.repository_validation.repository_checks_audit.scenario.fail_closed]
- GIVEN a missing or unexpected row, classification or rationale drift, unsafe or stale evidence, a selected marker, missing-set drift, dangling markers, or successful coverage
- WHEN maintained audit validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept the audit

### Requirement: Repository-check evidence completion
r[lattice.repository_validation.repository_checks_completion] Lattice MUST maintain direct adversarial evidence for all 11 reviewed repository-validation rows without promoting selected markers or changing unrelated linkage.

#### Scenario: Check-scoped signing remains external
r[lattice.repository_validation.repository_checks_completion.scenario.signing]
- GIVEN a host-level missing signing key and the maintained full-check command
- WHEN validation runs with the command-scoped empty `secret-key-files` override
- THEN validation MUST proceed without a placeholder key and the flake MUST contain no signing-policy override

#### Scenario: Nested and non-Rust sources remain available
r[lattice.repository_validation.repository_checks_completion.scenario.source_closure]
- GIVEN a configured test path dependency with nested output-like Rust names and a tracked JSON fixture
- WHEN Git ignore classification and filtered Nix Nextest or Clippy compilation run
- THEN the nested Rust source and JSON fixture MUST remain admitted, compiled, and directly observed

#### Scenario: Formatter and Clippy ownership remain bounded
r[lattice.repository_validation.repository_checks_completion.scenario.ownership]
- GIVEN the compiled path dependency is intentionally nonconforming to project formatting and Clippy policy
- WHEN flake format and Clippy checks run
- THEN they MUST check package `lattice`, Clippy MUST deny repository-owned warnings, and dependency lint admission MUST remain disabled

#### Scenario: Exactly 11 rows become exact
r[lattice.repository_validation.repository_checks_completion.scenario.exact]
- GIVEN the reviewed 315-item inventory and the 11-ID repository-validation tranche
- WHEN completion validation runs
- THEN exactly those 11 rows MUST be exact, BLAKE3-bound to current safe evidence, and marker-free while every unrelated row remains unchanged

#### Scenario: Repository-check completion drift fails closed
r[lattice.repository_validation.repository_checks_completion.scenario.fail_closed]
- GIVEN command, signing, source-filter, ignore, fixture, formatter, Clippy, classification, evidence, marker, missing-set, dangling, or successful-coverage drift
- WHEN maintained completion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept completion

### Requirement: Repository-check evidence promotion
r[lattice.repository_validation.repository_checks_promotion] Lattice MUST promote only the 11 reviewed exact repository-validation rows through markers attached to maintained assertions and MUST preserve every unrelated missing or dangling linkage state.

#### Scenario: Exact 11-ID repository-check delta
r[lattice.repository_validation.repository_checks_promotion.scenario.delta]
- GIVEN the failing 315-item completion receipt and the explicit 11-ID repository-validation allowlist
- WHEN maintained promotion validation compares current failing coverage
- THEN baseline minus current MUST equal the allowlist, current minus baseline MUST be empty, and missing coverage MUST equal 304

#### Scenario: Repository-check markers remain locally bound
r[lattice.repository_validation.repository_checks_promotion.scenario.bindings]
- GIVEN every selected row has exact maintained signing, source-closure, formatter, or Clippy assertions
- WHEN its marker is promoted
- THEN each marker MUST attach directly to its required test and MUST NOT rely on configuration names, file scope, generated inventory, lifecycle tests, or archived evidence

#### Scenario: Repository-check promotion drift fails closed
r[lattice.repository_validation.repository_checks_promotion.scenario.fail_closed]
- GIVEN a duplicate allowlist or receipt ID, missing binding, stale reviewed row, unexpected removal or addition, dangling marker, or successful coverage
- WHEN maintained promotion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept promotion

### Requirement: Agent-runtime traceability audit
r[lattice.repository_validation.agent_runtime_audit] Lattice MUST maintain a bounded explicit review of all 14 remaining agent-runtime debt rows without changing behavior or promoting reviewed identifiers.

#### Scenario: Complete agent-runtime partition
r[lattice.repository_validation.agent_runtime_audit.scenario.complete]
- GIVEN the 14 missing agent-runtime identifiers and an explicit expectation table
- WHEN maintained audit validation runs
- THEN each identifier MUST have exactly one non-`unassessed` classification and no unrelated row MUST be classified by this audit

#### Scenario: Agent-runtime evidence strength remains explicit
r[lattice.repository_validation.agent_runtime_audit.scenario.evidence]
- GIVEN direct compatibility, effect, fabric, authority, output, and reasoning evidence
- WHEN evidence bindings are checked
- THEN exact rows MUST cover every clause, partial rows MUST name their direct gap, evidence MUST bind current safe bytes, and no selected marker may be added

#### Scenario: Agent-runtime audit drift fails closed
r[lattice.repository_validation.agent_runtime_audit.scenario.fail_closed]
- GIVEN row, classification, rationale, evidence, marker, missing-set, dangling, or successful-coverage drift
- WHEN maintained audit validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept the audit

### Requirement: Agent-runtime evidence completion
r[lattice.repository_validation.agent_runtime_completion] Lattice MUST maintain exact direct evidence for all 14 reviewed agent-runtime rows without promoting selected markers or changing unrelated linkage.

#### Scenario: Partial runtime boundaries become directly observed
r[lattice.repository_validation.agent_runtime_completion.scenario.boundaries]
- GIVEN missing-route, credential, provider-failure, identity, authority-bearing output, incomplete-status, and unsupported-input boundaries
- WHEN maintained positive and negative tests run
- THEN each boundary MUST directly assert its accepted state, effect, redaction, output, or zero-dispatch clause

#### Scenario: Planning remains pure and shells remain bounded
r[lattice.repository_validation.agent_runtime_completion.scenario.core_shell]
- GIVEN explicit program identity inputs and runtime transport/report effects
- WHEN completion evidence exercises them
- THEN deterministic admission MUST remain testable without transport and shell assertions MUST observe exact effect counts and terminal mapping

#### Scenario: Exactly 14 rows are exact
r[lattice.repository_validation.agent_runtime_completion.scenario.exact]
- GIVEN the reviewed 304-item inventory and 14-ID agent-runtime tranche
- WHEN completion validation runs
- THEN all 14 rows MUST be exact, current-byte-bound, marker-free, and leave every unrelated row unchanged

#### Scenario: Agent-runtime completion drift fails closed
r[lattice.repository_validation.agent_runtime_completion.scenario.fail_closed]
- GIVEN classification, rationale, evidence, marker, missing-set, dangling, or successful-coverage drift
- WHEN maintained completion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept completion

### Requirement: Agent-runtime evidence promotion
r[lattice.repository_validation.agent_runtime_promotion] Lattice MUST promote only the 14 reviewed exact agent-runtime rows through markers attached to maintained assertions and MUST preserve every unrelated linkage state.

#### Scenario: Exact 14-ID agent-runtime delta
r[lattice.repository_validation.agent_runtime_promotion.scenario.delta]
- GIVEN the failing 304-item completion receipt and explicit 14-ID allowlist
- WHEN promotion validation compares current failing coverage
- THEN baseline minus current MUST equal the allowlist, current minus baseline MUST be empty, and missing MUST equal 290

#### Scenario: Agent-runtime markers remain locally bound
r[lattice.repository_validation.agent_runtime_promotion.scenario.bindings]
- GIVEN every selected row has exact maintained assertions
- WHEN markers are promoted
- THEN each marker MUST attach to its required integration, internal shell, or OpenAI test and MUST NOT rely on file scope or lifecycle evidence

#### Scenario: Agent-runtime promotion drift fails closed
r[lattice.repository_validation.agent_runtime_promotion.scenario.fail_closed]
- GIVEN duplicate, missing-binding, stale-policy, removal, addition, dangling, or successful-coverage drift
- WHEN promotion validation runs
- THEN it MUST return deterministic issue codes and MUST NOT accept promotion

### Requirement: Workflow-validation traceability audit
r[lattice.repository_validation.workflow_validation_audit] Lattice MUST maintain a byte-bound classification audit for every selected missing workflow-validation requirement without promoting Tracey coverage.

#### Scenario: Complete workflow-validation classification
r[lattice.repository_validation.workflow_validation_audit.scenario.complete]
- GIVEN the current workflow-validation missing subset
- WHEN the audit is checked
- THEN every selected ID MUST have exactly one explicit exact, partial, or no-exact decision

#### Scenario: Workflow-validation evidence identity
r[lattice.repository_validation.workflow_validation_audit.scenario.evidence]
- GIVEN a reviewed decision cites maintained evidence
- WHEN the audit is checked
- THEN every safe relative evidence path MUST match its recorded BLAKE3 file hash

#### Scenario: Workflow-validation audit drift fails closed
r[lattice.repository_validation.workflow_validation_audit.scenario.fail_closed]
- GIVEN classification, evidence, marker, or coverage drift
- WHEN the audit is checked
- THEN deterministic validation MUST reject the drift

### Requirement: Workflow-validation evidence completion
r[lattice.repository_validation.workflow_validation_completion] Lattice MUST maintain direct assertions that close every gap identified by the workflow-validation audit before promotion.

#### Scenario: Runtime policy behavior completion
r[lattice.repository_validation.workflow_validation_completion.scenario.behavior]
- GIVEN a runner declares an effect unsupported by its selected runtime
- WHEN admission runs
- THEN validation MUST report a deterministic fatal diagnostic before runtime execution

#### Scenario: Complete workflow-validation evidence
r[lattice.repository_validation.workflow_validation_completion.scenario.evidence]
- GIVEN the audited evidence gaps
- WHEN maintained tests run
- THEN positive and negative assertions MUST cover every accepted clause

#### Scenario: Exact workflow-validation classification
r[lattice.repository_validation.workflow_validation_completion.scenario.classification]
- GIVEN completed maintained evidence
- WHEN inventory is checked
- THEN all 29 reviewed workflow-validation rows MUST be exact while coverage remains unchanged and unmarked

### Requirement: Workflow-validation evidence promotion
r[lattice.repository_validation.workflow_validation_promotion] Lattice MUST promote the exact reviewed workflow-validation tranche only through assertion-local markers.

#### Scenario: Exact workflow-validation bindings
r[lattice.repository_validation.workflow_validation_promotion.scenario.bindings]
- GIVEN 29 exact reviewed IDs
- WHEN markers are added
- THEN each selected ID MUST be bound directly to an allowlisted maintained test

#### Scenario: Exact workflow-validation delta
r[lattice.repository_validation.workflow_validation_promotion.scenario.delta]
- GIVEN the immutable 290-gap baseline
- WHEN current coverage is checked
- THEN exactly the selected IDs MUST be removed, none added, zero markers dangling, and coverage MUST remain unsuccessful

#### Scenario: Workflow-validation promotion drift fails closed
r[lattice.repository_validation.workflow_validation_promotion.scenario.fail_closed]
- GIVEN marker, policy, or coverage drift
- WHEN promotion validation runs
- THEN deterministic validation MUST reject the drift

### Requirement: Agent-orchestration traceability audit
r[lattice.repository_validation.agent_orchestration_audit] Lattice MUST maintain a byte-bound classification audit for all 39 selected missing agent-orchestration requirements without promotion.

#### Scenario: Complete agent-orchestration classification
r[lattice.repository_validation.agent_orchestration_audit.scenario.complete]
- GIVEN the selected missing subset
- WHEN audit validation runs
- THEN every ID MUST have exactly one explicit exact or partial decision

#### Scenario: Agent-orchestration evidence identity
r[lattice.repository_validation.agent_orchestration_audit.scenario.evidence]
- GIVEN reviewed maintained evidence
- WHEN hashes are checked
- THEN every safe path MUST match its current BLAKE3 bytes

#### Scenario: Agent-orchestration audit drift fails closed
r[lattice.repository_validation.agent_orchestration_audit.scenario.fail_closed]
- GIVEN decision, evidence, marker, or coverage drift
- WHEN validation runs
- THEN the audit MUST reject it deterministically

### Requirement: Agent-orchestration evidence completion
r[lattice.repository_validation.agent_orchestration_completion] Lattice MUST maintain direct exact evidence for all 39 audited agent-orchestration rows before promotion.

#### Scenario: Complete bounded orchestration behavior
r[lattice.repository_validation.agent_orchestration_completion.scenario.behavior]
- GIVEN the six explicit audit gaps
- WHEN maintained pure-core tests run
- THEN inert output, overage, status, non-claim, and cycle boundaries MUST be directly asserted

#### Scenario: All agent-orchestration rows exact
r[lattice.repository_validation.agent_orchestration_completion.scenario.exact]
- GIVEN completed maintained evidence
- WHEN inventory is generated
- THEN all 39 rows MUST be exact, byte-bound, and unmarked while coverage remains unchanged

#### Scenario: Agent-orchestration completion drift fails closed
r[lattice.repository_validation.agent_orchestration_completion.scenario.fail_closed]
- GIVEN evidence, classification, marker, or coverage drift
- WHEN completion validation runs
- THEN it MUST reject deterministically

### Requirement: Agent-orchestration evidence promotion
r[lattice.repository_validation.agent_orchestration_promotion] Lattice MUST promote exactly the 39 completed agent-orchestration rows through assertion-local markers.

#### Scenario: Exact agent-orchestration bindings
r[lattice.repository_validation.agent_orchestration_promotion.scenario.bindings]
- GIVEN exact maintained tests
- WHEN markers are promoted
- THEN every selected ID MUST be bound to an allowlisted test

#### Scenario: Exact agent-orchestration delta
r[lattice.repository_validation.agent_orchestration_promotion.scenario.delta]
- GIVEN the immutable 261-gap baseline
- WHEN coverage is compared
- THEN exactly 39 selected IDs MUST be removed, none added, zero dangling remain, and coverage MUST stay unsuccessful

#### Scenario: Agent-orchestration promotion drift fails closed
r[lattice.repository_validation.agent_orchestration_promotion.scenario.fail_closed]
- GIVEN binding, policy, or set drift
- WHEN promotion validation runs
- THEN it MUST reject deterministically

### Requirement: Workflow-format traceability audit
r[lattice.repository_validation.workflow_format_audit] The repository MUST maintain a byte-bound classification audit for all 41 selected missing workflow-format requirements without promotion.

#### Scenario: Complete classification
r[lattice.repository_validation.workflow_format_audit.scenario.complete]
- GIVEN the selected missing subset
- WHEN audit validation runs
- THEN every ID MUST have exactly one explicit exact or partial decision

#### Scenario: Evidence identity
r[lattice.repository_validation.workflow_format_audit.scenario.evidence]
- GIVEN reviewed maintained evidence
- WHEN hashes are checked
- THEN every safe path MUST match its current BLAKE3 bytes

#### Scenario: Fail-closed drift
r[lattice.repository_validation.workflow_format_audit.scenario.fail_closed]
- GIVEN decision, evidence, marker, or coverage drift
- WHEN validation runs
- THEN the audit MUST reject it deterministically

### Requirement: Workflow-format evidence completion
r[lattice.repository_validation.workflow_format_completion] Lattice MUST maintain direct exact evidence for all 41 audited workflow-format rows before promotion.

#### Scenario: Complete workflow-format boundaries
r[lattice.repository_validation.workflow_format_completion.scenario.behavior]
- GIVEN the five explicit audit gaps
- WHEN maintained positive and negative tests run
- THEN image import, malformed control flow, opaque credentials, and cookbook drift MUST be directly asserted

#### Scenario: All workflow-format rows exact
r[lattice.repository_validation.workflow_format_completion.scenario.exact]
- GIVEN completed maintained evidence
- WHEN inventory is generated
- THEN all 41 rows MUST be exact, byte-bound, and unmarked while missing coverage remains unchanged

#### Scenario: Workflow-format completion drift fails closed
r[lattice.repository_validation.workflow_format_completion.scenario.fail_closed]
- GIVEN evidence, classification, marker, or coverage drift
- WHEN completion validation runs
- THEN it MUST reject deterministically

### Requirement: Workflow-format evidence promotion
r[lattice.repository_validation.workflow_format_promotion] Lattice MUST promote exactly the 41 completed workflow-format rows through assertion-local markers.

#### Scenario: Exact workflow-format bindings
r[lattice.repository_validation.workflow_format_promotion.scenario.bindings]
- GIVEN exact maintained tests
- WHEN markers are promoted
- THEN every selected ID MUST bind to an allowlisted test

#### Scenario: Exact workflow-format delta
r[lattice.repository_validation.workflow_format_promotion.scenario.delta]
- GIVEN the immutable 222-gap completion baseline
- WHEN current coverage is compared
- THEN exactly 41 selected IDs MUST be removed, none added, zero dangling remain, and missing coverage MUST equal 181

#### Scenario: Workflow-format promotion drift fails closed
r[lattice.repository_validation.workflow_format_promotion.scenario.fail_closed]
- GIVEN binding, policy, duplicate, or set drift
- WHEN promotion validation runs
- THEN it MUST reject deterministically

### Requirement: Bounded Octet baseline migration
r[lattice.repository_validation.octet_baseline_migration] Lattice MUST migrate accepted Octet identities only when the bounded semantic finding policy accepts the complete transition.

#### Scenario: Dynamic counts retain semantic identity
r[lattice.repository_validation.octet_baseline_migration.scenario.dynamic]
- GIVEN assertion-density or function-length messages differ only in dynamic line counts
- WHEN migration compares findings
- THEN it MUST preserve the function-scoped semantic identity

#### Scenario: Exact resolved finding is allowed
r[lattice.repository_validation.octet_baseline_migration.scenario.resolved]
- GIVEN the sole old-only finding is the reviewed assertion-density finding for `workflow_diagnostic_from_runner`
- WHEN migration runs
- THEN it MAY remove that accepted finding

#### Scenario: Semantic drift fails closed
r[lattice.repository_validation.octet_baseline_migration.scenario.fail_closed]
- GIVEN a candidate-only finding or any unapproved old-only finding
- WHEN migration runs
- THEN it MUST reject without writing the baseline

### Requirement: Workflow-execution traceability audit
r[lattice.repository_validation.workflow_execution_audit] Lattice MUST maintain a byte-bound classification audit for all 83 selected missing workflow-execution requirements without promotion.

#### Scenario: Complete classification
r[lattice.repository_validation.workflow_execution_audit.scenario.complete]
- GIVEN the selected missing subset
- WHEN audit validation runs
- THEN every ID MUST have exactly one explicit exact or partial decision

#### Scenario: Evidence identity
r[lattice.repository_validation.workflow_execution_audit.scenario.evidence]
- GIVEN reviewed maintained evidence
- WHEN hashes are checked
- THEN every safe path MUST match its current BLAKE3 bytes

#### Scenario: Fail-closed drift
r[lattice.repository_validation.workflow_execution_audit.scenario.fail_closed]
- GIVEN decision, evidence, marker, or coverage drift
- WHEN validation runs
- THEN the audit MUST reject deterministically

### Requirement: Workflow-execution evidence completion
r[lattice.repository_validation.workflow_execution_completion] Lattice MUST maintain direct exact evidence for all 83 audited workflow-execution rows before promotion.

#### Scenario: Pinned Rats source closure
r[lattice.repository_validation.workflow_execution_completion.scenario.rats]
- GIVEN the repository-owned Rats sync boundary
- WHEN source-closure and focused consumer checks run
- THEN the exact pinned revision, required nested sources, generated snapshots, and offline compilation MUST be proved

#### Scenario: All workflow-execution rows exact
r[lattice.repository_validation.workflow_execution_completion.scenario.exact]
- GIVEN completed maintained evidence
- WHEN inventory is generated
- THEN all 83 rows MUST be exact, byte-bound, and unmarked while missing coverage remains unchanged

#### Scenario: Workflow-execution completion drift fails closed
r[lattice.repository_validation.workflow_execution_completion.scenario.fail_closed]
- GIVEN evidence, classification, marker, correlation, or coverage drift
- WHEN completion validation runs
- THEN it MUST reject deterministically

### Requirement: Exact workflow-execution evidence promotion
r[lattice.repository_validation.workflow_execution_promotion] Lattice MUST promote exactly the 83 completed workflow-execution requirements through assertion-local markers and an exact missing-set transition.

#### Scenario: Exact workflow-execution delta
r[lattice.repository_validation.workflow_execution_promotion.scenario.delta]
- GIVEN the completion baseline with 181 missing requirements
- WHEN workflow-execution evidence is promoted
- THEN current coverage MUST contain 98 missing requirements, remove exactly the 83 allowlisted IDs, add no ID, retain zero dangling markers, and remain `valid: false`

#### Scenario: Workflow-execution markers remain assertion-local
r[lattice.repository_validation.workflow_execution_promotion.scenario.bindings]
- GIVEN the reviewed workflow-execution evidence files and exact binding table
- WHEN promoted marker bindings are validated
- THEN each identifier MUST occur immediately before its named supporting test and MUST be absent from reviewed debt policy

#### Scenario: Workflow-execution promotion drift fails closed
r[lattice.repository_validation.workflow_execution_promotion.scenario.fail_closed]
- GIVEN duplicate or unsorted missing sets, duplicate allowlists, missing bindings, stale policy rows, unrelated removals, additions, dangling markers, or successful coverage
- WHEN deterministic promotion validation runs
- THEN it MUST return explicit issue codes and MUST NOT accept the promotion

### Requirement: Remaining TUI/UX traceability audit
r[lattice.repository_validation.tui_ux_traceability_audit] Lattice MUST classify the exact remaining 98 TUI/UX requirements against bounded maintained evidence without changing Tracey linkage.

#### Scenario: Exact TUI/UX audit scope
r[lattice.repository_validation.tui_ux_traceability_audit.scenario.scope]
- GIVEN the failing 98-item Tracey receipt after workflow-execution promotion
- WHEN the TUI/UX audit runs
- THEN every missing ID MUST appear exactly once and no non-missing ID may enter the audit

#### Scenario: Byte-bound TUI/UX evidence
r[lattice.repository_validation.tui_ux_traceability_audit.scenario.evidence]
- GIVEN bounded repository-owned evidence snapshots
- WHEN an audit row cites evidence
- THEN its safe relative path and current file-byte BLAKE3 hash MUST match the reviewed snapshot

#### Scenario: Honest TUI/UX classification
r[lattice.repository_validation.tui_ux_traceability_audit.scenario.classification]
- GIVEN direct assertions and identified behavioral gaps
- WHEN evidence strength is classified
- THEN each row MUST be `exact_maintained_evidence` or `partial_evidence_only` with an explicit bounded rationale and no inventory row may satisfy the requirement

#### Scenario: TUI/UX audit drift fails closed
r[lattice.repository_validation.tui_ux_traceability_audit.scenario.fail_closed]
- GIVEN a missing row, duplicate row, unsafe path, hash drift, invalid classification, changed missing set, dangling marker, successful coverage, or selected marker
- WHEN deterministic audit validation runs
- THEN it MUST return explicit issue codes and MUST NOT accept the audit

### Requirement: TUI/UX evidence completion
r[lattice.repository_validation.tui_ux_evidence_completion] Lattice MUST complete every partial row from the bounded TUI/UX audit before any selected requirement is promoted.

#### Scenario: All TUI/UX rows become exact
r[lattice.repository_validation.tui_ux_evidence_completion.scenario.exact]
- GIVEN the immutable audit with 81 exact and 17 partial rows
- WHEN bounded TUI behavior and assertions are completed
- THEN all 98 rows MUST be exact against current safe relative evidence paths and current file-byte BLAKE3 hashes

#### Scenario: Completion preserves Tracey debt
r[lattice.repository_validation.tui_ux_evidence_completion.scenario.coverage]
- GIVEN exactly 98 missing TUI/UX requirements and zero dangling markers
- WHEN completion evidence is recorded
- THEN the missing set MUST remain unchanged, coverage MUST remain unsuccessful, and no selected marker may be attached

#### Scenario: TUI/UX completion drift fails closed
r[lattice.repository_validation.tui_ux_evidence_completion.scenario.fail_closed]
- GIVEN a stale partial classification, missing row, unsafe evidence path, byte-hash drift, selected marker, changed missing set, dangling marker, or successful coverage
- WHEN deterministic completion validation runs
- THEN it MUST return explicit issue codes and MUST NOT accept completion

### Requirement: Exact final TUI/UX evidence promotion
r[lattice.repository_validation.tui_ux_promotion] Lattice MUST promote exactly the 98 completed TUI/UX requirements through assertion-local markers and an exact final missing-set transition.

#### Scenario: Exact final TUI/UX delta
r[lattice.repository_validation.tui_ux_promotion.scenario.delta]
- GIVEN the completion baseline with 98 missing requirements and unsuccessful coverage
- WHEN TUI/UX evidence is promoted
- THEN current coverage MUST contain zero missing requirements, remove exactly the 98 allowlisted IDs, add no ID, retain zero dangling markers, and be `valid: true`

#### Scenario: TUI/UX markers remain assertion-local
r[lattice.repository_validation.tui_ux_promotion.scenario.bindings]
- GIVEN the reviewed TUI/UX evidence files and exact binding table
- WHEN promoted marker bindings are validated
- THEN each identifier MUST occur immediately before its named supporting test and MUST be absent from reviewed debt policy

#### Scenario: Final TUI/UX promotion drift fails closed
r[lattice.repository_validation.tui_ux_promotion.scenario.fail_closed]
- GIVEN duplicate or unsorted missing sets, duplicate allowlists, missing bindings, stale policy rows, incomplete final coverage, unrelated removals, additions, dangling markers, or unsuccessful final coverage
- WHEN deterministic final-promotion validation runs
- THEN it MUST return explicit issue codes and MUST NOT accept the promotion

### Requirement: Reproducible bounded-exec source identity
r[lattice.repository_validation.bounded_exec_source] Lattice MUST consume `bounded-exec` only from one reviewed immutable published Git revision, MUST align Cargo and Nix source identity to that revision, and MUST reject sibling path substitution, source drift, or an unresolvable dependency.

#### Scenario: Published immutable source is admitted
r[lattice.repository_validation.bounded_exec_source.scenario.published]
- GIVEN the canonical `bounded-exec` repository exposes the reviewed revision and its AGPL source
- WHEN Cargo and Nix resolve Lattice's dependency closure
- THEN both MUST use that exact revision and repository validation MUST confirm the aligned identity

#### Scenario: Unpublished or substituted source fails closed
r[lattice.repository_validation.bounded_exec_source.scenario.invalid]
- GIVEN the canonical source is unpublished, the revision cannot resolve, Cargo and Nix identities differ, or a sibling path is substituted
- WHEN repository validation evaluates the dependency
- THEN it MUST fail before build or release evidence and MUST NOT treat the local checkout as a durable product input

### Requirement: Post-effect crash-boundary evidence
r[lattice.repository_validation.post_effect_crash_boundaries] Lattice MUST continuously exercise scheduler and webhook process death after an observable workflow effect but before terminal persistence, MUST compare pre-restart and post-restart durable facts against a reviewed deterministic observation profile, and MUST report at-least-once risk without claiming exactly-once external effects.

#### Scenario: Scheduler restart exposes repeated-effect risk
r[lattice.repository_validation.post_effect_crash_boundaries.scenario.scheduler]
- GIVEN a due scheduled workflow records an observable effect and then terminates the scheduler before terminal run persistence or schedule advancement
- WHEN the crashed store is inspected without repairing the original and the scheduler restarts at the same due time
- THEN the check MUST prove the schedule remained due, MUST observe the reviewed repeated effect attempt, and MUST find only the restarted terminal run in durable history

#### Scenario: Webhook redelivery is characterized
r[lattice.repository_validation.post_effect_crash_boundaries.scenario.webhook]
- GIVEN an admitted webhook workflow records an observable effect and then terminates the listener before its terminal run, trigger-history, or HTTP response boundary is durably complete
- WHEN the crashed store is inspected and the same trigger-scoped idempotency key is redelivered after restart
- THEN the check MUST compare the response, effect-attempt count, run records, and trigger history to the version-controlled webhook crash profile and MUST fail on an unreviewed outcome

#### Scenario: Earlier death does not masquerade as post-effect evidence
r[lattice.repository_validation.post_effect_crash_boundaries.scenario.missing-effect]
- GIVEN a forced-death fixture terminates before the observable effect checkpoint
- WHEN crash observations are classified
- THEN the check MUST reject the fixture as insufficient evidence and MUST NOT report a post-effect crash result

#### Scenario: Exactly-once remains outside the claim
r[lattice.repository_validation.post_effect_crash_boundaries.scenario.non-claim]
- GIVEN a crash profile observes one or more local effect attempts and corresponding Lattice durable records
- WHEN the result is documented or emitted by continuous validation
- THEN it MUST state that the evidence does not prove remote commit identity, provider deduplication, external cancellation, arbitrary kill safety, or exactly-once external execution
