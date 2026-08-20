# Verification evidence

Date: 2026-08-20

## Baseline

The existing `lattice-workflow-contract` suite passed before implementation:

- 16 unit tests
- 2 producer-fixture tests
- 2 source-boundary tests

The repository's live Tracey coverage and inventory checks already failed at `origin/main` revision `66fe7b9b43567490caacc797ebda307e734c4419`.

The baseline verdicts were `tracey_status=1` and `inventory_status=1`. The baseline reported the existing dangling and expected-missing policy mismatch.

## Focused transport evidence

The nested workflow workspace passed all targets and all features:

- `lattice-workflow-contract`: 20 tests
- `lattice-workflow-transport`: 17 tests
- strict Clippy with `-D warnings`: pass
- format check: pass

The transport tests cover deterministic identities, closed operations, bounded frames, stale controls, exact logs, peer admission, peer denial, deadlines, malformed responses, disconnect ambiguity, producer fixtures, and source boundaries.

The pinned `workflow-transport-octet` Nix check passed with zero findings, zero warnings, and zero errors.

The focused Nix test, Clippy, format, and strict Octet derivations passed.

## Nickel evidence

`contracts/workflow-transport-profile.ncl` typechecks.

The positive profile exports successfully. Negative profiles for zero frame bytes, empty peers, and unsorted peers fail contract evaluation.

## Lifecycle evidence

Cairn proposal, design, and task gates passed. Cairn validation passed.

Cairn sync updated `.cairn/specs/workflow-interchange/spec.md`. The local accepted-spec test then passed all three cases.

The final Tracey report contains no `lattice.workflow_interchange` item in its missing or dangling sets. The repository still retains the pre-existing unrelated Tracey and inventory debt recorded in the baseline.

## Repository gates

- `scripts/check.rs`: pass
- `nix flake check -L --option secret-key-files ''`: pass

One concurrent full run exposed an unrelated analyzer timing failure. The isolated Nextest derivation then passed all 1,024 tests.

A later full run exposed an unrelated process-spawn failure. The isolated Lattice derivation passed, and the final full flake check passed.

## Claim boundary

This evidence covers local protocol shape, deterministic identity, contract linkage, bounded framing, Linux peer observation, and tested failure classification.

It does not prove peer intent, workflow correctness, runtime availability, operation authority, host confinement, external effect success, or production deployment.
