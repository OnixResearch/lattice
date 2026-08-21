# Admitted execution backend orchestration validation

## Baseline

Before implementation, the proposal, design, and task gates passed. The operation-contract, operation-CLI, and store test suites also passed.

The existing operation path selected a process runtime and persisted an invocation start. It did not bind a complete backend-neutral requirement profile or one exact durable backend route.

## Implementation result

Lattice now owns typed execution requirements for format, filesystem, network, process, terminal, peripheral, persistence, isolation, and positive resource bounds.

The operation catalog and invocation plan use version 2 schemas. Their identities bind the requirement profile and a frozen projection of the Animus planner contract at revision `54b2fdaf57ac9bbc34e6c73f75d803d625f84ec0`.

Positive and negative Nickel fixtures accept the reviewed cohort and reject a stale cohort. Lattice maps the reviewed backend classes but retains runtime, adapter, authority, credential-reference, budget, persistence, recovery, and effect admission.

The durable run start now contains the selected route, run identity, and attempt identity before dispatch. Terminal evidence preserves an `observed` or `unknown` backend-attempt state.

Retry admission denies route substitution. Unknown outcomes require explicit settlement before replanning. CLI JSON includes metadata-only route evidence, and TUI run details expose only class, identity, and state.

## Validation

The following checks passed:

- focused positive and negative Nickel fixture evaluation;
- 18 operation-contract tests;
- operation CLI, operation-pack, workflow-format, and store tests;
- repeated debug and release truncation-boundary tests;
- `cargo clippy --package lattice --all-targets --no-deps -- -D warnings`;
- `scripts/check.rs`, including operator E2E and the full Cargo suite;
- `octet-precommit-check` with zero new and zero stale findings;
- the Octet identity migration positive and negative self-test;
- `nix build .#checks.x86_64-linux.nextest --no-link -L --max-jobs 1 --cores 2`, with 1,027 tests passed and one skipped; and
- `nix flake check -L --option secret-key-files '' --builders '' --max-jobs 1 --cores 2`, with all checks passed.

Tracey still reports pre-existing repository debt. None of the seven `lattice.execution_backend_orchestration.*` requirement IDs are missing.

The Octet migration replaced two file-level identities for the unchanged `workflow_format.rs` file-length and filename findings. It preserved all 855 accepted semantic findings and added no lint category.

The Nix suite exposed a scheduler-sensitive analyzer truncation producer. A compiled one-write test helper replaced that shell-only fixture. Product truncation classification did not change.

## Claim boundary

This evidence establishes selected requirement, identity, persistence, drift-denial, recovery, inspection, and fixture facts.

It does not prove backend availability, sandbox enforcement, provider correctness, effect success, work completion, output correctness, production readiness, or release eligibility.
