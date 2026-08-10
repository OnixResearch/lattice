# Design: Maintain the project identity audit

## Context

Tracey reports 698 accepted requirements, 329 referenced requirements, 369 missing requirements, zero dangling markers, and `valid: false`. The two missing `lattice.project_identity` scenarios have markers only in `cairn/archive/2026-07-20-rename-n8nclone-to-lattice/evidence/check-identity.rs`, outside the checked evidence roots.

The archived checker cannot simply be copied. It exempts all of `tests/test_identity.rs`, `src/config/identity.rs`, and the project-identity spec, and its Cairn-history marker does not inspect archive or accepted-ID immutability.

## Goals and non-goals

### Goals

- Make stale-branding and Cairn-history checks maintained test behavior.
- Keep classification deterministic and independently testable without filesystem I/O.
- Permit only explicit compatibility, stable protocol, and durable marker identities.
- Detect changes to historical `n8nclone.*` accepted IDs and original rename-archive bytes with BLAKE3.
- Close exactly the two target missing IDs while retaining zero dangling markers and fail-closed coverage.

### Non-goals

- Do not rewrite stable `n8nclone` protocol, state, requirement, marker, or archive identities.
- Do not scan imported dependencies, build outputs, active Cairn changes, or unrelated archives as public branding.
- Do not prove arbitrary Git history, external publication state, semantic correctness, or release readiness.

## Decisions

### Decision: separate the pure audit core from filesystem collection

`tests/identity_audit/core.rs` owns deterministic line classification, canonical-surface checks, accepted-ID extraction, and BLAKE3 manifest construction over supplied snapshots. `tests/identity_audit/shell.rs` only walks bounded roots in sorted order and reads repository files. `tests/test_identity.rs` orchestrates assertions.

**Rationale:** Classification and mutation behavior can be tested with in-memory fixtures, while live repository tests remain thin and reviewable.

### Decision: classify complete lines instead of exempting files

The public scan covers `Cargo.toml`, `README.md`, `flake.nix`, `src`, `tests`, `examples`, and `scripts`. A line containing case-insensitive `n8nclone` is allowed only when every occurrence is one of:

- a marker-only `r[...]` token on a line containing no prose;
- an exact stable webhook or vault domain in its owning source file; or
- an exact legacy config, environment, data-directory, or store token in `src/config/identity.rs`.

The existing binary-help negative assertion and traceability guard constants construct legacy identities from fragments so the audit does not need to exempt their test files. Synthetic negative fixtures do the same at runtime.

**Rationale:** Whole-file exemptions are hidden authority. Exact path-and-token admission makes every compatibility exception explicit and keeps tests self-auditing.

### Decision: bind historical continuity to two pinned BLAKE3 manifests

The history test hashes deterministic, length-delimited snapshots for:

1. every sorted `path + n8nclone.* accepted requirement ID` extracted from `cairn/specs`; and
2. every sorted path and byte sequence under the original rename archive.

The expected hashes are named constants. A negative test mutates each manifest input and requires a different digest. The archive itself remains untouched.

**Rationale:** The accepted scenario requires unchanged identities and recorded archive bytes. A content manifest proves those local facts more directly than the archived checker's canonical-name assertions.

### Decision: include bounded audit inputs in isolated test sources

The Nix source filter retains the public `README.md`, `flake.nix`, and `scripts/` audit surfaces in addition to existing Cargo sources and fixtures. It admits only `cairn/archive/2026-07-20-rename-n8nclone-to-lattice/` as historical audit input, not every archive.

**Rationale:** Nix tests must observe the same current public surfaces and pinned evidence without expanding package inputs to unrelated historical material.

## Bounded search registry

| Family | Mechanism | Result | Reason |
|---|---|---|---|
| Copy archived checker | Move the one-off script unchanged into current tests | Rejected | Broad file exemptions create blind spots and Cairn-history is not actually checked. |
| Add markers to existing binary test | Attach both missing IDs without new audit behavior | Rejected | The binary-help test proves neither repository-wide stale branding nor archive continuity. |
| Maintained classified audit | Pure line classifier plus bounded shell and pinned BLAKE3 manifests | Selected | It supplies positive and adversarial evidence for the exact accepted scenarios. |

An adversarial review emphasized self-exemption, marker-comment bypass, and manifest mutation tests. The final design scans tests rather than exempting them, admits only marker-only lines, and requires negative mutations for public text, accepted IDs, and archive bytes.

## Validation

- Baseline and final Tracey receipts with exact non-target missing-set comparison.
- Focused identity tests, including positive and negative cases.
- Policy freshness, `scripts/check.rs`, and bounded Nix flake checks.
- Cairn proposal, design, tasks, validation, sync, and archive gates.
