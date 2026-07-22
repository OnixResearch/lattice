# Operator-diagnostics traceability audit

## Scope and non-claims

This evidence covers a bounded semantic review of the eight missing `operator-diagnostics` identifiers in the maintained debt inventory. It records evidence strength without adding their Tracey markers or changing doctor behavior.

An exact inventory classification is not requirement satisfaction. A partial classification is not a near-complete correctness claim. `no_exact_evidence_identified` records the result of this bounded maintained-source audit and does not prove global behavioral absence. The review does not prove semantic correctness, complete traceability, release readiness, production readiness, provider truth, or external evidence anchoring.

## Portfolio and adversarial audit

Four mechanisms were evaluated:

1. Trusting archived completed tasks was rejected because archive claims are not maintained evidence.
2. Lexical matching was rejected because doctor names and output tokens do not establish complete scenarios.
3. Binary exact/absent classification was rejected because it erases meaningful partial evidence.
4. Requirement-by-requirement comparison against maintained source and tests was selected.

A local VibeThinker advisory review independently agreed with the conservative partition. Deterministic policy validation, file identities, tests, and coverage comparison—not model agreement—remain authoritative.

## Baseline

Before the audit, `cairn traceability coverage --root .` exited unsuccessfully with:

- requirements: 706
- referenced: 345
- missing: 361
- dangling: 0
- `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `8e2f5b622c9fd99677e5187a71f8fcbdb3fa5ee763ae34bc053ffb8c731c9a02`
- receipt BLAKE3: `22637d7584cc23085a3d40b992451e7b4d8622f989ddf2baeeb5c723cce269af`

The previous promotion receipt at `tests/fixtures/traceability-candidate-promotion-current.json` is the checked baseline for exact missing-set comparison.

## Classification result

### Exact maintained evidence: 2

- `n8nclone.operator_diagnostics.doctor_core`
  - rationale: `source.pure-deterministic-evaluate`
  - evidence: `src/main.rs`
  - bounded finding: `evaluate` is a pure deterministic transformation that sanitizes facts, sorts findings by severity and ID, and derives status without I/O.
- `n8nclone.operator_diagnostics.doctor_output`
  - rationale: `source.deterministic-dual-renderers`
  - evidence: `src/main.rs`
  - bounded finding: maintained text and JSON presenters deterministically render the ordered typed report.

### Partial evidence only: 4

- `n8nclone.operator_diagnostics.doctor_core.fatal`
  - rationale: `assertions.enumerated-failures-incomplete`
  - evidence: `src/main.rs`, `tests/test_doctor.rs`
  - gap: tests do not cover every enumerated failure or assert all stable fields and remediation hints; listener collection emits only healthy facts.
- `n8nclone.operator_diagnostics.doctor_core.healthy`
  - rationale: `assertions.healthy-findings-indirect`
  - evidence: `src/main.rs`, `tests/test_doctor.rs`
  - gap: the CLI test asserts healthy status but not the complete finding set directly.
- `n8nclone.operator_diagnostics.doctor_output.json`
  - rationale: `assertions.mixed-json-output-missing`
  - evidence: `src/main.rs`, `tests/test_doctor.rs`
  - gap: no mixed healthy-warning-fatal deterministic JSON assertion exists.
- `n8nclone.operator_diagnostics.doctor_output.text`
  - rationale: `assertions.mixed-text-output-missing`
  - evidence: `src/main.rs`, `tests/test_doctor.rs`
  - gap: no mixed healthy-warning-fatal deterministic text assertion exists.

### No exact evidence identified: 2

- `n8nclone.operator_diagnostics.doctor_redaction`
  - rationale: `behavior.redaction-inputs-unwired`
  - evidence: none
  - gap: fact constructors leave redactions empty, no shell path populates them, and the existing text sentinel is never injected.
- `n8nclone.operator_diagnostics.doctor_redaction.json`
  - rationale: `assertions.json-redaction-missing`
  - evidence: none
  - gap: redaction inputs remain unwired and no JSON redaction test exists.

## Evidence identities

- `src/main.rs`: `e36262a8289e29d552560b429af2603456d32ca9766b32223caec2e5e4ed1a8d`
- `tests/test_doctor.rs`: `12b75ea05dae3a639f4f024dce632ee853a345e2ddc9c13753ccdde41eb68086`

Exact and partial rows bind these current file bytes. No-exact rows bind no files.

## Maintained inventory

- rows: 361
- reviewed rows: 8
- exact rows: 2
- partial rows: 4
- no-exact rows: 2
- unassessed rows: 353
- missing-set BLAKE3: `df059bfa0b70e75996de6dc41990d0d744197b90bee9195bf1b873c505abaf3c`
- review-policy BLAKE3: `2ce38b700b3de64ce77c596a2c94c98da83018fef25bc2159a9ba14f5c289a71`
- inventory BLAKE3: `03e2d418ee7ded71db5cc629004da6fa0bbb6ca71e1ff4b35884722e962a40be`
- issues: none
- inventory `valid: true`

## Coverage invariant

The new bounded tranche-review requirement and its three scenarios add four accepted and four referenced IDs. Final maintained coverage still exits unsuccessfully with:

- requirements: 710
- referenced: 349
- missing: 361
- dangling: 0
- `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `4aacd4fc05dbd5e6fd515e4b4b7e1515f4aca882058012cfcba4324465b4b00d`
- receipt BLAKE3: `2d65d23ff28e90be6f93656d5e9a26a049f38038da486c2f13768e2adbf1e4cf`

`tests/traceability_review/core.rs` proves the baseline and final missing sets are equal, both dangling arrays are empty, and both receipts remain invalid. The positive test also verifies that none of the eight selected IDs gained maintained source/test markers.

## Positive and negative validation

- Baseline doctor tests: 4 passed.
- Baseline inventory tests: 6 passed.
- Tranche-review tests: 2 passed.
- The positive test validates the complete expectation table, exact classification counts, rationale codes, evidence paths, current BLAKE3 file bytes, unchanged coverage, and absent selected markers.
- The negative test rejects a missing row, unexpected row, classification drift, rationale drift, evidence-path drift, removed/added/duplicate missing IDs, dangling markers, unsorted input, and successful coverage with deterministic issue codes.
- The maintained identity audit initially rejected literal legacy branding in the new test; compatibility IDs were changed to split path-owned protocol tokens and the final identity suite passes.
- `scripts/traceability-inventory.rs generate` and `check` agree byte-for-byte.
- `scripts/check.rs` passes repository-owned formatting, the complete test suite, strict project-only Clippy, identity checks, and Cairn spec checks.
- Cairn policy freshness, repository validation, proposal/design/tasks gates, and sync pass.
- `nix flake check -L --option secret-key-files ''` passes every configured `x86_64-linux` check; incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems are omitted.
- Octet retains 913 accepted/current findings with zero new and zero stale IDs under baseline `b3:ec5609b38db008f4145f4c38490b04290d8b52266e7b5a067d2cb29751ac208f`. Its underlying Cargo subprocess remains classified as exit code 101 while the reviewed zero-delta wrapper accepts it.
