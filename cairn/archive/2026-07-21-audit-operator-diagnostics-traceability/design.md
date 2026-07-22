## Context

After the reviewed safe-inspection promotion, maintained Cairn coverage reports 706 requirements, 345 referenced requirements, 361 missing identifiers, zero dangling identifiers, and `valid: false`. Eight missing rows belong to `operator-diagnostics`. The doctor implementation is concentrated in a private module inside `src/main.rs`; its shell behavior is covered by four integration tests in `tests/test_doctor.rs`.

The bounded audit found complete source evidence for the pure deterministic evaluation core and the existence of deterministic text/JSON presenters. Healthy, fatal, and mixed-output scenarios have only partial assertions. Redaction is not exact: `Fact` constructors leave `redactions` empty, no shell path populates them, the text sentinel is never injected, and no JSON redaction test exists.

## Goals

- Classify all eight operator-diagnostics rows exactly once.
- Preserve distinctions between exact, partial, and no-exact evidence.
- Bind exact and partial evidence to safe repository-relative paths and current file-byte BLAKE3 hashes.
- Keep no-exact rows free of evidence references and avoid claiming behavioral absence beyond the bounded audit.
- Preserve the exact 361-item missing array, zero dangling markers, and unsuccessful Tracey coverage.
- Add deterministic positive and negative tests for this review tranche.

## Non-Goals

- Adding operator-diagnostics Tracey markers.
- Implementing missing warning, listener-failure, or redaction behavior.
- Treating archived doctor change tasks as current evidence.
- Claiming that partial evidence satisfies accepted requirements.
- Claiming semantic correctness, complete traceability, release readiness, or production readiness.

## Portfolio Search Contract

**Exact goal:** classify the eight currently missing operator-diagnostics rows without changing their coverage state.

**Completion evidence:** all eight IDs have explicit non-`unassessed` decisions; classifications equal the audited 2 exact / 4 partial / 2 no-exact partition; exact/partial rows bind current maintained file hashes; no-exact rows bind no files; all other inventory rows remain unchanged; before/after missing arrays and dangling arrays are equal; coverage remains invalid.

**False completion:** promoting markers, using archive checkboxes as evidence, treating source-token similarity as scenario proof, calling partial evidence exact, attaching evidence to no-exact rows, omitting an ID, reviewing an unrelated row, changing the missing set, or reporting successful coverage.

**Budgets:** accepted operator-diagnostics and repository-validation specs; `src/main.rs`; `tests/test_doctor.rs`; current inventory policy/tool/report; one local advisory review; no network retrieval; one implementation round and one adversarial validation round.

**Allowed outcomes:** validated classification, an exact bounded blocker, or user decision if evidence authority is ambiguous.

## Approach Registry

| Family | Mechanism | State | Reason |
|---|---|---|---|
| archive acceptance | Trust completed historical tasks as current evidence | falsified | Archive claims do not establish maintained source or test behavior. |
| lexical matching | Infer exactness from doctor names and output tokens | falsified | Tokens do not prove complete GIVEN/WHEN/THEN assertions. |
| binary classification | Mark every row exact or absent | falsified | It erases meaningful partial evidence and overstates absence. |
| bounded semantic review | Compare each accepted requirement with maintained source/tests and record exact, partial, or no-exact | selected | It preserves evidence strength and uncertainty without changing coverage. |

## Adversarial Audit

A local VibeThinker review independently agreed with the proposed partition: exact for `doctor_core` and `doctor_output`; partial for healthy/fatal core scenarios and text/JSON output scenarios; no-exact for redaction parent and JSON scenario. The decisive defects are absent direct core assertions, incomplete enumerated fatal cases, no mixed warning report path, unwired redaction lists, a non-injected text sentinel, and no JSON redaction test.

The advisory result is not authority. Typed policy validation, file identities, deterministic tranche tests, and exact coverage comparison remain authoritative.

## Decisions

### Decision: exact rows require complete maintained implementation evidence

`doctor_core` binds `src/main.rs` because `evaluate` is a pure deterministic transformation that sanitizes, sorts, and derives status without I/O. `doctor_output` binds the same source because both presenters deterministically render the ordered typed report.

### Decision: scenario evidence remains partial when assertions omit accepted clauses

Healthy core evidence does not directly assert the complete finding set. Fatal core evidence omits several enumerated failures and stable-field/remediation checks. Text/JSON output evidence lacks a mixed healthy-warning-fatal report and warning fact construction. These rows bind both source and doctor tests but remain `partial_evidence_only`.

### Decision: redaction rows record no exact evidence

The current redaction mechanism only replaces caller-supplied strings, but no caller supplies them. The existing text test checks a sentinel that was never injected, and JSON redaction is untested. Both rows use `no_exact_evidence_identified` with empty evidence. This is a bounded evidence conclusion, not proof that no relevant behavior exists anywhere.

### Decision: review validation is pure

A test-only pure core compares the generated inventory rows with an explicit expectation table and validates classification, rationale, evidence paths, unexpected reviewed rows in the selected subsystem, and unchanged coverage sets. Test shells only deserialize checked-in receipts/reports and hash compile-time file bytes.

## Risks and Mitigations

- **Risk:** broad `src/main.rs` identity obscures exact symbols. **Mitigation:** rationale codes identify the reviewed mechanism, tests pin file bytes, and no marker is promoted.
- **Risk:** a partial row is mistaken for satisfied coverage. **Mitigation:** inventory non-claims remain explicit and Tracey stays missing.
- **Risk:** no-exact is read as behavioral absence. **Mitigation:** rationale and non-claims scope the conclusion to this bounded maintained-source audit.
- **Risk:** future source drift invalidates reviewed evidence. **Mitigation:** exact/partial rows bind BLAKE3 file hashes and inventory checks fail closed.

## Validation

- Run baseline doctor and inventory tests before policy changes.
- Add positive and negative pure tranche-review tests.
- Regenerate and byte-check the inventory against final failing coverage.
- Compare exact baseline/final missing and dangling arrays.
- Run repository checks, policy freshness, Cairn validation/gates, Octet, and `nix flake check -L --option secret-key-files ''`.
