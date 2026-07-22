# Finished operator-diagnostics evidence

## Scope and success contract

This evidence closes the eight operator-diagnostics rows reviewed in the preceding bounded audit. Success requires executable exact assertions, a real shell-level secret non-disclosure check, an exact eight-ID Tracey removal, no added missing IDs, zero dangling markers, and still-unsuccessful repository coverage.

Markers alone, archived task claims, lexical matches, a secret sentinel that was never supplied, successful commands without exact assertions, or a green inventory were excluded as completion evidence.

## Baseline

Before this change, maintained coverage reported:

- requirements: 710
- referenced: 349
- missing: 361
- dangling: 0
- `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `4aacd4fc05dbd5e6fd515e4b4b7e1515f4aca882058012cfcba4324465b4b00d`
- receipt BLAKE3: `2d65d23ff28e90be6f93656d5e9a26a049f38038da486c2f13768e2adbf1e4cf`
- missing-set BLAKE3: `df059bfa0b70e75996de6dc41990d0d744197b90bee9195bf1b873c505abaf3c`

The binary had no doctor-core unit tests. Four doctor CLI tests passed before core changes.

## Portfolio result

Four mechanisms were evaluated:

- Embedded tests were sufficient behaviorally but rejected because they left a non-trivial core inside the shell file.
- CLI-only tests were falsified as complete evidence because the CLI cannot safely synthesize arbitrary mixed-severity and secret-bearing facts.
- A public library API was rejected as stronger than the goal because it creates an unnecessary compatibility surface.
- A binary-local functional core plus bounded CLI leak tests was selected.

A VibeThinker advisory review also favored extraction over integration-only testing. Its assertion that sanitization during evaluation was inherently insufficient was rejected after direct sensitive-input tests proved the evaluated `Report` is sanitized before either renderer. Deterministic repository checks remain authoritative.

## Functional core and shell

The deterministic doctor model, fact-to-finding evaluation, status reduction, ordering, sanitization, and rendering now live in `src/doctor/core.rs`. `src/main.rs` retains config, filesystem, environment, store, process output, and orchestration.

The core:

- orders fatal before warning before healthy and then by stable identifier;
- sanitizes identifiers, messages, and remediation text before constructing a `Finding`;
- removes empty redaction inputs, sorts redactions longest-first with a lexical tie-break, and deduplicates before replacement;
- omits the sensitive redaction list from `Report` and JSON; and
- renders exact deterministic text and JSON from the sanitized report.

Longest-first replacement prevents a shorter overlapping value from leaving a suffix of a longer sensitive value.

The adversarial pass identified a separate shell leak route: raw Nickel parser/evaluator diagnostics could echo malformed source values. Doctor config loading now emits fixed bounded failure categories instead of backend debug values. A malformed Nickel fixture containing the secret sentinel proves both text and JSON failures omit it. A second CLI test supplies an actual environment value to a credential-backed webhook and proves stdout and stderr in both formats omit that value.

Current evidence file identities:

- `src/doctor/core.rs`: `1adcbf27ee2fc97ee0b4ec4dabb58b9b04c90bb3dcc7eae41646c0edfb11554d`
- `tests/test_doctor.rs`: `3458911c03fd9367aaf0c1087acba70d71ec508a4d9d4df54914d6cd970b3a91`
- `src/main.rs`: `e676a71694763d30b16182759eec2cee5eb4fb4dbaa9fd4bee2ba85c16173d5e`

## Exact reclassification

Before promotion, `tests/fixtures/traceability-operator-diagnostics-reviewed.json` records all eight rows as `exact_maintained_evidence` against current maintained file bytes:

- reviewed: 8
- exact: 8
- partial: 0
- no-exact: 0
- unassessed: 353
- issues: none
- inventory `valid: true`
- review-policy BLAKE3: `a5fd56c1751fda3e897ff42dbe675f006ad21d976ca239f72c40e5040de58da6`
- inventory BLAKE3: `d82ba20fc30de6eb50d04678bc02c13c5773abbf1f0809ac7292813cec9ea5cf`

The exact assertions cover healthy evaluation, every enumerated fatal category, mixed healthy/warning/fatal text, equivalent mixed JSON, plaintext secret values, environment values, authorization headers, raw payloads, provider outputs, overlapping redactions, and absent redaction metadata.

## Promotion result

The eight promoted identifiers are exactly:

- `n8nclone.operator_diagnostics.doctor_core`
- `n8nclone.operator_diagnostics.doctor_core.fatal`
- `n8nclone.operator_diagnostics.doctor_core.healthy`
- `n8nclone.operator_diagnostics.doctor_output`
- `n8nclone.operator_diagnostics.doctor_output.json`
- `n8nclone.operator_diagnostics.doctor_output.text`
- `n8nclone.operator_diagnostics.doctor_redaction`
- `n8nclone.operator_diagnostics.doctor_redaction.json`

Every marker is attached immediately to its exact test. The maintained comparator proves baseline minus current equals this allowlist, with no additions and no dangling markers.

Final maintained coverage reports:

- requirements: 710
- referenced: 357
- missing: 353
- dangling: 0
- `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `8eac7421fbd9f64acda404ddf4936ce5268dbdad22ef876ac06392e293fcaaf5`
- receipt BLAKE3: `72e6ac2a7fae5933c17631506f54d38f7016203054b3e619f66f739319e35c06`
- missing-set BLAKE3: `0e205e16faa2c958a0ba9f90bb64bbc8d09c81c16f8dd93f9798fd958a4835e1`

## Maintained inventory

After promotion, the eight stale review decisions were removed and the current report was regenerated:

- rows: 353
- reviewed: 0
- unassessed: 353
- dangling: 0
- coverage `valid: false`
- inventory issues: none
- inventory `valid: true`
- review-policy BLAKE3: `d07125dc5aab4e510aecb7471e946514d39b3b995b6c248bf799fffe1868463f`
- inventory BLAKE3: `75fc950452ef4c493fa4f9637de540d0ef160fd709bb95ef3921c10c4277537d`

Inventory validity does not satisfy any remaining requirement.

## Validation

- 5 doctor-core unit tests pass.
- 5 doctor CLI tests pass.
- 2 exact-reclassification tests pass.
- 3 promotion tests pass, including adversarial transition mutations.
- 6 inventory tests pass.
- 2 marker-alignment tests pass.
- Strict project-only Clippy initially rejected test-only warning/redaction builders as production dead code; those helpers are now test-gated and strict Clippy passes.
- `scripts/check.rs` passes repository formatting, all tests, strict project-only Clippy, identity checks, and Cairn spec checks.
- Deterministic inventory generation and checking agree.
- The strengthened redaction requirement was synced into the accepted operator-diagnostics specification, and Cairn policy freshness, repository validation, and proposal/design/tasks gates pass.
- Octet retains 913 accepted/current findings with zero new and zero stale IDs under baseline `b3:ec5609b38db008f4145f4c38490b04290d8b52266e7b5a067d2cb29751ac208f`. Its underlying Cargo process remains classified as exit code 101 while the reviewed zero-delta wrapper accepts it.
- `nix flake check -L --option secret-key-files ''` passes all configured `x86_64-linux` checks. Incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems are omitted.

## Non-claims

This result proves only the checked local core transformations, bounded CLI observations, evidence identities, and exact traceability transition. It does not prove confidentiality against arbitrary host observers, external provider readiness, remote credential behavior, complete diagnostics, whole-system correctness, complete traceability, release readiness, or production readiness.
