## Why

Lattice currently uses the execution-graph behavior extracted from Seaglass. Choregraph will provide the single formally verified machine implementation, canonical lowering, replay decisions, and effect intents.

Lattice is the first consumer integration, but it must retain product contracts, evaluator shell, runtime registry, scheduling, persistence, retries, triggers, diagnostics, effects, and evidence authority.

## What Changes

- Require the accepted execution-graph and Nickel evaluator baselines before Choregraph migration.
- Verify exact Choregraph source, license, release bundle, proofs, canonical artifacts, frontend boundary, and conformance evidence.
- Map Lattice workflow values into Choregraph's pure lowerer and machine core.
- Dual-run graph admission, readiness, lifecycle, resume, replay, and effect-intent decisions.
- Migrate persisted machine-state and event mappings without replaying historical effects.
- Cut production decisions over to Choregraph and remove duplicate Lattice and compatibility logic after parity.
- Preserve a tested VCS rollback with no hidden runtime or source fallback.

## Success Criteria

- The reviewed Lattice diamond graph lowers and executes through Choregraph's pure decision path.
- Every inventoried current behavior remains compatible or has a reviewed migration.
- Persisted state and replay mappings fail closed on unsupported or stale identities.
- Lattice retains all product and runtime authority.
- Production contains one transition implementation.

## Non-Goals

- Claiming Choregraph proves Lattice as a whole.
- Moving Nickel source discovery, runtime effects, persistence, retries, scheduling, or product diagnostics into Choregraph.
- Publishing or pushing Choregraph or consumer repositories.

## Impact

- **Dependencies**: exact Choregraph source in Cargo and Nix after upstream evidence passes.
- **Source**: workflow lowering, runner core adapter, state and event mapping, and replay boundary.
- **Testing**: complete dual-run matrix, persisted compatibility, downstream fixtures, source checks, and rollback.
