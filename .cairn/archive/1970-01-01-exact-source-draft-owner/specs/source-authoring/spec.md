## ADDED Requirements

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
