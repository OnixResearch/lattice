## Context

The reviewed core credential inventory records 12 exact rows, zero partial rows, and 341 unassessed rows while coverage remains 353 missing with zero dangling markers. The exact rows bind credential, executor, workflow-format, workflow-validation, and credential-test file bytes.

## Goals

- Promote exactly the 12 reviewed credential scenario IDs.
- Bind every marker directly to an exact supporting test.
- Preserve every unrelated missing ID and add no missing or dangling ID.
- Remove promoted rows from the debt policy.
- Keep global coverage invalid after the exact 353-to-341 transition.

## Non-Goals

- Changing credential behavior.
- Promoting any of the other 14 credential gaps.
- Treating parent markers, file scope, archived evidence, or lexical similarity as scenario evidence.
- Claiming complete credential traceability, release readiness, or production readiness.

## Portfolio Search Contract

**Exact goal:** transform the maintained missing set by removing only the explicit 12-ID credential allowlist.

**Completion evidence:** baseline minus current equals the allowlist; current minus baseline is empty; baseline/current arrays are sorted and duplicate-free; dangling arrays are empty; both receipts remain invalid; every marker is attached to an allowlisted exact test; no promoted ID remains in policy.

**False completion:** broad parent marker promotion, duplicate allowlist entries, a missing allowlisted removal, an unrelated removal, an added missing ID, a dangling marker, stale reviewed policy rows, or successful global coverage.

**Budgets:** current exact review fixture, credential tests, generic pure promotion validator, one new focused promotion test, one coverage round, and repository/Cairn/Octet/Nix gates.

**Allowed outcomes:** validated exact promotion, exact blocker, or user decision if an accepted ID no longer matches its reviewed scenario.

## Decisions

Combined exact tests may carry multiple scenario markers when one maintained assertion set proves both accepted outcomes. The plaintext rejection scenario is attached to both the typed shape test and the CLI no-store test because the complete accepted outcome spans both assertions.

The promotion validator remains a pure set transformation over decoded receipts. The shell only loads checked-in fixtures and source text. Negative mutations cover duplicates, ordering, missing expected removals, unrelated removals, additions, dangling markers, and incorrectly successful coverage.
