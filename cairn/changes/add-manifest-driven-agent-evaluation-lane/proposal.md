## Why

Lattice can run bounded agents, immutable workflows, durable orchestration, local process adapters, and restart-aware evidence. Animus supplies pure Harbor-compatible evaluation contracts.

The stack still lacks one concrete product evaluation lane. There is no typed catalog that binds user-level agent tasks, environments, hard gates, rubrics, artifacts, and live deployment profiles.

camelAI demonstrates a useful manifest-driven evaluation workflow. Its catalog separates unit, skill, hard, and real-deploy cases and emits one standard result envelope.

Lattice will adapt this workflow through Nickel, Rust, BLAKE3, Animus, Cairn, explicit authority, and metadata-only receipts. It will not copy camelAI's Cloudflare runner or raw transcript defaults.

## What Changes

- Add a typed Nickel agent-evaluation catalog as the only source for committed case discovery and execution planning. r[lattice.agent_evaluation.catalog]
- Define versioned `unit`, `skill`, `hard`, and `live` case classes with immutable subjects, environments, routes, budgets, gates, rubrics, references, artifacts, and non-claims. r[lattice.agent_evaluation.case_contract]
- Add a pure matrix planner and resume decision over validated catalog data, exact repository or workload revisions, selected agent profiles, and environment profiles. r[lattice.agent_evaluation.planning]
- Add a thin host runner over injected workspace, agent, workflow, deployment, verifier, artifact, and clock services. r[lattice.agent_evaluation.execution]
- Keep deterministic persisted-state, safety, harness-integrity, and artifact-contract validations as hard gates. Keep semantic rubric judgment separate and advisory. r[lattice.agent_evaluation.oracles]
- Require frozen reference evidence and ground-truth identities that agents cannot author or mutate during a trial. r[lattice.agent_evaluation.oracles]
- Add explicit live-case authority, isolated deployment targets, unknown-outcome handling, retention policy, and no automatic promotion. r[lattice.agent_evaluation.live]
- Emit one standard metadata envelope plus separately authorized bounded artifacts without making raw transcripts normal telemetry. r[lattice.agent_evaluation.evidence]
- Add positive and negative local, hard, and live fixtures with focused CLI, restart, redaction, and claim-boundary evidence. r[lattice.agent_evaluation.verification]

## Success Criteria

- Catalog export and Rust validation discover exactly the committed evaluation cases.
- A case cannot run when its subject, agent, environment, policy, gate, rubric, reference, artifact, or budget identity drifts.
- Deterministic hard gates cannot be overridden by a judge score or agent prose.
- A live case cannot deploy without explicit live authority and an isolated admitted target.
- Standard receipts contain safe identities, classes, counts, outcomes, and non-claims only.
- Private artifacts remain allowlisted, bounded, identity-checked, and separate from receipts.
- Positive and negative cases pass through the production planner and injected runner shell.

## False Completion

A list of test file names is not a catalog. A generic batch runner without task, environment, gate, rubric, and artifact identities is not completion.

A judge-only score, agent-authored ground truth, raw transcript receipt, or deployment without explicit live authority is false completion.

## Impact

- **Lattice core**: pure catalog validation, matrix planning, resume admission, hard-gate reduction, and safe evidence projection.
- **Lattice shell**: injected local and live services, bounded execution, artifact persistence, and operator commands.
- **Animus**: existing Harbor evaluation, role, trajectory, and receipt contracts remain adapter inputs, not Lattice runtime dependencies by ambient path.
- **Configuration**: typed Nickel is authoritative. Generated runtime data is revalidated by Rust.
- **Testing**: local deterministic cases, fake live services, isolated deployment smoke profiles, restart cases, leak rejection, and focused Cairn and Nix rails.

## Out of Scope

- A cloud scheduler, public result viewer, training system, benchmark leaderboard, automatic prompt optimization, or remote marketplace.
- Raw transcript collection by default, hidden-test disclosure, provider billing truth, or automatic model promotion.
- Claims for verifier soundness, judge correctness, model quality, deployment correctness, production readiness, or release eligibility.

## Affected Specs

- `agent-evaluation`: catalog, case contract, planning, execution, oracles, live cases, evidence, and verification.
