# Design

## Goal and completion evidence

Finish the eight reviewed operator-diagnostics gaps with exact maintained assertions, then promote only those eight identifiers. Completion requires:

- a pure binary-local doctor core that can be tested without filesystem, process, environment, store, or network setup;
- complete healthy and enumerated fatal finding assertions;
- exact mixed healthy/warning/fatal text and JSON assertions;
- text and JSON tests proving secret, environment, authorization-header, provider-output, and raw-payload values are absent;
- a CLI test that injects a real environment value and proves neither output format discloses it;
- an exact eight-identifier missing-set removal, no added missing IDs, zero dangling markers, and still-unsuccessful coverage; and
- regenerated debt inventory state with no stale reviewed decisions.

Markers, archived claims, lexical similarity, successful commands without assertions, or a sentinel that was never supplied are false completion.

## Portfolio registry

### Embedded unit tests

- Family: existing-module hardening.
- Mechanism: keep the pure module inside `main.rs` and add local tests.
- Claim: closes the behavior gaps with the smallest diff.
- Blocker: leaves a non-trivial functional core embedded in the imperative shell file.
- State: rejected in favor of an equally bounded explicit core module.

### CLI-only tests

- Family: black-box integration.
- Mechanism: synthesize doctor states only through configuration, store, credentials, and triggers.
- Claim: validates operator-visible behavior.
- Blocker: the CLI cannot safely construct a mixed healthy/warning/fatal report or arbitrary secret-bearing facts, so output and redaction-core assertions would remain indirect.
- State: falsified as complete evidence; retained only as a shell-level leak check.

### Public library extraction

- Family: reusable library API.
- Mechanism: expose doctor facts, evaluation, and rendering from the `lattice` library.
- Claim: allows integration tests and downstream reuse.
- Blocker: creates an unnecessary public API and compatibility surface for a command-local diagnostic model.
- State: rejected as stronger than the goal.

### Binary-local functional core

- Family: functional core plus imperative shell.
- Mechanism: move the deterministic model/evaluation/rendering code to `src/doctor/core.rs`, re-export it only to the binary shell through `src/doctor/mod.rs`, and keep filesystem/environment/store/process work in `main.rs`.
- Claim: makes every disputed behavior directly testable without creating a public library contract.
- Artifact: pure core tests plus a bounded CLI environment-value leak test.
- State: selected.

A VibeThinker advisory review favored extraction over integration-only tests. Its claim that post-evaluation masking is inherently insufficient was not accepted: deterministic sanitization during fact-to-finding evaluation is before both renderers and can be directly falsified with sensitive inputs. Repository tests remain authoritative.

## Core behavior

`Fact` gains deliberate warning and redaction construction. Redaction candidates are normalized deterministically by removing empty values, sorting longest-first with a lexical tie-break, and deduplicating before replacement. Longest-first replacement prevents a shorter overlapping value from leaving a suffix of a longer sensitive value.

`evaluate` remains pure: it converts facts to sanitized findings, orders fatal before warning before healthy and then by stable identifier, and derives the report status. Findings do not retain the sensitive redaction list.

Text and JSON rendering consume only sanitized `Report` values. The shell continues to check environment-variable presence without resolving or copying the value into findings.

## Test binding

Markers will be placed directly on complete tests, not on the module or file:

- healthy and fatal core tests bind evaluation requirements;
- mixed text and JSON tests bind renderer requirements;
- a redaction test binds the redaction requirement and both output scenarios;
- a CLI test injects a real environment value and checks both formats as shell-level defense.

Positive tests assert exact structure and output. Negative assertions prove fatal status, sensitive-value absence, overlap resistance, and no redaction metadata in JSON.

## Promotion invariant

The baseline is the current failing 361-ID receipt. A maintained promotion comparator will allowlist exactly the eight operator-diagnostics IDs. The final receipt must contain 353 missing requirements, no additions, no dangling markers, and `valid: false`. The inventory review decisions are removed only after those IDs cease to be missing, then the deterministic report is regenerated.

## Non-claims

Passing tests and coverage prove only the checked local transformations, shell observation, and exact marker-set transition. They do not prove host confidentiality, remote credential-provider behavior, arbitrary diagnostic completeness, release readiness, production readiness, or whole-system correctness.
