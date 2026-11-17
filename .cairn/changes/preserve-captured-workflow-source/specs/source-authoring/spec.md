## ADDED Requirements

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
