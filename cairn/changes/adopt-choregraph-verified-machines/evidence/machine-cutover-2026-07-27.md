# Lattice Choregraph machine cutover — 2026-07-27

## Baselines

- Lattice rollback revision: `c8d56ad83a57287064edb85f58808f53e4bf4536`
- accepted execution-graph lifecycle: `cairn/archive/2026-07-18-adopt-seaglass-execution-graph/`
- independent execution-graph Choregraph consumer: `8aafdae79940d948e8b9f0cc71c385a2fbf28988`
- accepted Nickel workflow import lifecycle: `cairn/archive/2026-06-03-add-nickel-workflow-import-export/`

The focused runner baseline passed 21 tests before dependency changes. An isolated rollback worktree at the recorded Lattice revision passed the same 21 tests.

## Source

```text
rid=rad:zL2ncTUeASVYwcoGkEXv9JKgGbAF
revision=fc47c5f4ecbb7b4341af690fa42199f25d57f54c
source-archive-blake3=dcb1faf95a7487145496ce986ae1639908600ace5bd77c4d96a37a530a7538a5
release-bundle-blake3=a2ac1336679542fa9573b356daddc18266fdda8147ddbe28f85aa37d135030d9
candidate-blake3=e4b5cd7bd77d78c20cf8b96cd0c038144028d2287000757c8ab68f5f5dfdff1a
```

Cargo aliases `choregraph-seaglass` as `seaglass-execution-graph` and directly selects `choregraph-core` for bounded replay. Nix and Crane override the exact Cargo source with the same locked Choregraph input. The former vendored execution-graph implementation is removed.

## Result

The post-cutover runner corpus passes the exact 21-test rail. Three additional migration tests cover all six persisted lifecycle states, stale workflow and machine bindings, unsupported schemas, foreign nodes, partial state, ambiguous state, finite replay, terminal denial, and stale replay bindings.

Replay returns `choregraph_core::ReplayDecision`, which has no effect-intent or dispatch field. Lattice runtime, policy, persistence, retry, trigger, diagnostic, UI, and evidence code does not import the Choregraph adapter.

## Claim boundary

This cutover proves exact source selection and the checked machine compatibility surface. It does not prove verifier soundness, whole-Lattice correctness, persistence correctness, effect safety, or release eligibility.
