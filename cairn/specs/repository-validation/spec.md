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
