## Implementation

- [x] [serial] Add a fixed typed DAG, shared bounded definition, trusted local rule producer and real build/test evaluator. r[lattice.agent_orchestration.bounded_build_loop]
- [x] [serial] Reuse orchestration transitions, durable store, exact operator profiles and the production snapshot route; retain actual round handoffs and immutable provenance. r[lattice.agent_orchestration.bounded_build_loop]
- [x] [serial] Add runtime positive/negative regression fixtures and document local authority/non-claims, construction versus execution/repetition, and disabled optional masked repair. r[lattice.agent_orchestration.bounded_build_loop]

## Evidence boundary

The worker ran no checks. Parent qualification executed the real local example: round one failed, round two passed, the second round consumed the first verdict handoff, and both rounds retained identical workflow and frozen-input identities. The observed report records two rounds and four agent invocations; its duration charge is a configured reservation, not measured execution time.

Native Cairn proposal, design and tasks gates passed with no issues, and
repository validation passed all 49 specifications. `scripts/check.rs` and
`octet-precommit-check` passed. The x86_64-linux Nix flake check passed all 32
checks; Nextest ran 1,438 passing tests with 9 skipped. Octet retained 1,056 root
and 25 core accepted IDs with no new or stale IDs. These local observations do
not qualify a remote model, distributed deployment, other platform or release.
