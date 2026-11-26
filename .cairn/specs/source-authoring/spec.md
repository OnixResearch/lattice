# Source Authoring Specification

## Purpose

Defines the `source-authoring` capability.

## Requirements

### Requirement: Truthful bounded captured source association

r[lattice.source_authoring.capture] Lattice MUST retain bounded exact captured entry/member bytes and explicit source-declared parameter overrides separately from the materialized workflow when capture is available. Legacy workflows MUST remain valid without fabricated provenance. Captured bytes MUST NOT be represented as a complete evaluated module closure when the selected evaluator lacks captured-only import resolution. Missing Choregraph construction origins MUST report unavailable.
Source preview/readmission MUST reject syntactic imports through the selected native parser until captured-only module resolution is available; comments and literal strings containing import-like text MUST NOT be mistaken for imports. Stored previews MUST pass current native validation and graph admission again at readmission.

#### Scenario: Retain and reopen author structure

r[lattice.source_authoring.capture.reopen]
- GIVEN an import-free program with a helper, source-defined instances and generated node collection
- WHEN its source-text preview is admitted, stored, reopened and source-exported
- THEN exact captured bytes and parameter identity MUST survive
- AND retained unused module bytes MUST remain distinguishable from observed import resolution.

### Requirement: Atomic exact preview admission

r[lattice.source_authoring.admission] Source preview MUST execute no workloads, deliver no credentials and grant no authority. Admission MUST compare exact durable draft, source/compiler/candidate identities and expected workflow revision atomically. Failed evaluation/admission MUST retain the previous admitted workflow and an editable failed draft with bounded category diagnostics. Durable catalog generations MUST prevent stale token reuse after delete/recreate.

#### Scenario: Concurrent and stale source drafts

r[lattice.source_authoring.admission.concurrent]
- GIVEN two previews based on the same admitted revision
- WHEN they attempt concurrent admission
- THEN at most one MUST replace that revision
- AND the other MUST remain a failed editable draft without overwriting the successful workflow or newer draft.

### Requirement: Explicit materialized lineage fork

r[lattice.source_authoring.fork] A materialized edit without supported source correspondence MUST NOT rewrite source. All materialized editing/persistence paths MUST apply one explicit fork rule preserving exact source lineage and marking that the unchanged source does not reproduce the edited graph. Existing workflow clones MUST retain their exact old source association and catalog revision.

#### Scenario: Materialized edit and old snapshot

r[lattice.source_authoring.fork.snapshot]
- GIVEN an admitted source-associated workflow and an existing clone
- WHEN the current workflow is edited and saved materially
- THEN its association MUST be a materialized fork with unchanged captured lineage
- AND the existing clone MUST retain its old graph, association and revision.

### Requirement: Separate exact source and expanded export

r[lattice.source_authoring.exports] Default export of an associated workflow MUST preserve exact captured author bytes in an explicitly identified source archive, not silently flatten them. Expanded export MUST be explicit and separately identified. Source archive output MUST require an unambiguous `.author.json` target; legacy source-free default export MUST remain materialized Nickel.

#### Scenario: Distinct binary command exports

r[lattice.source_authoring.exports.cli]
- GIVEN a workflow admitted through the registered source CLI
- WHEN ordinary export targets `.ncl` without requesting expanded output
- THEN it MUST reject the misleading archive filename before creating that target
- AND `.author.json` source export and explicit `--expanded` Nickel export MUST preserve their distinct artifact meanings.

### Requirement: Exact owner source view

r[lattice.source_edit.view] The Lattice owner command MUST expose versioned machine-readable head inspection and a read-only view that compares the caller's expected catalog head and returns the exact selected captured member and observed draft fingerprint or absence from one snapshot. It MUST reject missing source, missing members, and drafts for a different workflow or head.

#### Scenario: View a selected exact member

- GIVEN a source-associated workflow and its current catalog revision
- WHEN the caller views a retained logical member at that revision
- THEN the response contains its exact bytes and current draft fingerprint or absence
- AND a different expected head is rejected without mutation.

### Requirement: Atomic validated draft-only save

r[lattice.source_edit.save] The owner MUST apply only the selected member replacement, validate the resulting exact program with the native preview, and atomically compare the supplied catalog and draft revisions before saving. A null expected draft revision MUST require absence. The route MUST NOT publish a workflow, manage authority, execute a workload or change state on a validation or CAS rejection.

#### Scenario: A concurrent writer changes the base

- GIVEN a candidate was natively validated outside the write transaction
- WHEN a competing writer changes the head or draft before commit
- THEN the command rejects the candidate without changing the current workflow or draft.

### Requirement: Native draft receipt

r[lattice.source_edit.receipt] The command MUST return the exact durable draft and source identities and its native preview compiler, evaluated output and candidate identities only after a successful draft commit. Rejections MUST be distinguishable and MUST NOT echo arbitrary evaluator output, source text or host paths.

#### Scenario: Draft receipt leaves the admitted graph unchanged

- GIVEN a valid edit to existing captured source
- WHEN native validation and both CAS comparisons succeed
- THEN the response binds the committed candidate and the unchanged admitted head
- AND reopening the draft returns its exact validated bytes without an admission.
