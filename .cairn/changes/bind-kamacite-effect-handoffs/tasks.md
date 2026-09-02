## Phase 1: Producer and dependency admission

- [ ] [serial] Select and record one immutable Kamacite semantic operation, compatibility, codec, and license cohort. r[lattice.choregraph_effect_authority.semantic_operation_profile]
- [ ] [serial] Require a published Basalt runtime-effect contract that binds the same semantic operation identity or record the exact blocker. r[lattice.choregraph_effect_authority.semantic_operation_authority]
- [ ] [depends:kamacite-operation-cohort] Define the reviewed Choregraph-to-Kamacite operation registry with exact source and descriptor identities. r[lattice.choregraph_effect_authority.semantic_operation_registry]

## Phase 2: Pure binding and durable attempts

- [ ] [depends:semantic-operation-registry] Add Lattice-owned nominal semantic operation and compatibility references at the inbound adapter boundary. r[lattice.choregraph_effect_authority.semantic_operation_profile]
- [ ] [depends:basalt-semantic-operation-contract] Bind the same semantic identity across Choregraph intent, Basalt request, policy decision, and final dispatch decision. r[lattice.choregraph_effect_authority.semantic_operation_authority]
- [ ] [depends:semantic-operation-authority] Persist semantic operation and compatibility identities before product adapter invocation. r[lattice.choregraph_effect_authority.semantic_operation_attempt]
- [ ] [depends:semantic-operation-authority] Admit only exact directional compatibility for the requested live context. r[lattice.choregraph_effect_authority.semantic_operation_compatibility]
- [ ] [depends:semantic-operation-attempt] Retain the recorded identity in success, failure, unknown, recovery, and abandonment observations. r[lattice.choregraph_effect_authority.semantic_operation_observation]

## Phase 3: Replay and fixtures

- [ ] [parallel] Preserve semantic identities during replay while denying registry resolution and product dispatch. r[lattice.choregraph_effect_authority.semantic_operation_replay]
- [ ] [parallel] Add one process and one Animus agent positive end-to-end fixture. r[lattice.choregraph_effect_authority.semantic_operation_verification]
- [ ] [parallel] Add negative missing-registry, ambiguous, name-equal, stale, reverse, replay-only, Basalt-mismatch, adapter-mismatch, and mixed-record fixtures. r[lattice.choregraph_effect_authority.semantic_operation_verification]

## Phase 4: Documentation and gates

- [ ] [serial] Document registry ownership, Basalt dependency, compatibility rules, persistence, recovery, replay denial, rollout, and non-claims. r[lattice.choregraph_effect_authority.semantic_operation_observation] r[lattice.choregraph_effect_authority.semantic_operation_replay]
- [ ] [depends:semantic-operation-verification] Run focused authority, adapter, persistence, replay, crash, Cairn, Clippy, Octet, and Nix checks. r[lattice.choregraph_effect_authority.semantic_operation_verification]
