# Agent-evaluation baseline, portfolio, and references

Date: 2026-08-14

## Success contract

The goal is a typed, identity-bound product evaluation lane. Completion requires one Nickel-owned catalog, strict Rust validation, pure planning and reduction, injected services, durable starts, explicit live authority, bounded artifacts, safe receipts, positive and negative production-entrypoint tests, lifecycle sync, and archive.

False completion includes a test-name list, script-only matrix, judge-only verdict, agent-authored ground truth, raw transcript receipt, fake success after an unknown effect, or a live path without explicit isolated authority.

Audit risks are identity drift, evaluator self-control, judge override, artifact escape, secret or hidden-material leakage, stale resume, service fallback, uncertain live effects, automatic retry, and unsupported quality claims.

The search budget is three mechanism families, twelve targeted source or reference reads, four implementation rounds, ten focused baseline suites plus one library baseline, no subagents, no live credentials, and no production targets. Allowed outcomes are validated, blocked, exhausted, or user-decision-required.

## Baseline

The following pre-change checks passed with `CARGO_TARGET_DIR=target/agent-eval-baseline`:

- `test_agent_nodes`: 29 tests;
- `test_animus_agent`: 9 tests;
- `test_local_process`: 21 tests;
- `test_orchestration_core`: 24 tests;
- `test_orchestration_shell`: 13 tests;
- `test_orchestration_store`: 6 tests;
- `test_workflow_templates`: 17 tests;
- `test_guarded_agent_templates`: 19 tests;
- `test_run_inspection`: 16 tests;
- library suite: 111 tests.

The orchestration restart test intentionally launched an unknown-phase negative child that failed. The owning outer test passed after confirming that rejection, and the complete suite passed.

## Approach registry

### Lattice-owned core and injected shell

- **Family:** product-owned evaluation lane.
- **Mechanism:** add narrow Lattice catalog, planning, oracle, artifact, live, receipt, and runner components. Reuse existing Lattice workflow, orchestration, budget, recovery, and template owners at their public boundaries.
- **Claim:** satisfies the full Lattice product contract without widening current Animus or sibling authority.
- **Artifact:** `src/agent_evaluation/`, typed Nickel policy, committed fixtures, focused tests, CLI, and one reviewed workflow bundle.
- **Evidence:** current Lattice already supplies Nickel loading, BLAKE3, redb, bounded agents, workflow templates, guarded rounds, and safe inspection.
- **Gap strength:** simpler than a repository-wide Animus snapshot upgrade.
- **Blocker:** none.
- **Next check:** focused catalog, planner, runner, live, artifact, and receipt tests.
- **State:** active.

### Upgrade and consume current Animus evaluation crates

- **Family:** published-component reuse.
- **Mechanism:** import `animus-eval-core`, `animus-adapter-eval`, and `animus-std-eval` from current Animus.
- **Claim:** reuses pure task, resume, service, and host contracts.
- **Artifact:** Animus revision `205b3d2c01ee310292a6e4ff2af0259a94779611` contains the components.
- **Evidence:** reviewed sibling docs and crate roots show no-std core, implementation-free ports, and a thin host shell.
- **Gap strength:** stronger dependency migration than this change requires.
- **Blocker:** Lattice pins and normalizes Animus revision `f1662a8ed290b170c3986d2ddf2909b36d7e5fdc`, whose selected snapshot excludes these crates. Updating it would mix a broad dependency-cohort migration into this product change.
- **Next check:** a separate pinned Animus cohort change can adopt these crates later.
- **State:** blocked for this change.

### Script or workflow-bundle-only lane

- **Family:** orchestration-only adaptation.
- **Mechanism:** add evaluation scripts or guarded workflow bundles without a product domain core.
- **Claim:** provides a runnable matrix quickly.
- **Artifact:** camelAI-style manifest and matrix runner.
- **Evidence:** the reference demonstrates useful discovery and matrix mechanics.
- **Gap strength:** weaker than the requested result.
- **Blocker:** it cannot establish exact case contracts, pure resume, final hard gates, artifact admission, durable unknown outcomes, or metadata-only receipts.
- **Next check:** none; the proposal names this as false completion.
- **State:** falsified.

## Reference decisions

The reviewed camelAI source is `qaml-ai/camelAI` commit `70dd57a6219d9a19b25840aebbf1105c63c3a5f2`, tree `02b5ed7cd9ad75d9ef86ce6039eb037a5ee460f9`, under MIT. Lattice adopts one manifest as discovery authority, `unit` and `skill` classes, optional hard tier, explicit real-deploy posture, bounded matrices, and a standard result shape.

Lattice rejects camelAI JSON authoring, raw transcript defaults, ambient environment discovery, random and wall-clock identities, automatic finalize retries for unknown mutations, Cloudflare deployment coupling, public report upload, and judge-primary completion semantics.

The current Lattice Animus snapshot remains revision `f1662a8ed290b170c3986d2ddf2909b36d7e5fdc`. Current sibling Animus revision `205b3d2c01ee310292a6e4ff2af0259a94779611` is a reviewed design reference only. Its Harbor, native evaluation, trajectory, adapter, host, and receipt boundaries do not become ambient runtime dependencies.
