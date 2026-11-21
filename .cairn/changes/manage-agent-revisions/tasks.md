## Phase 1: Pure core

- [x] [serial] 1.1 Add `crates/lattice-core/src/agent_revision/` with the bounded lifecycle state, closed refusals, bundle facts, the components framing extension, lifecycle identities and port facts. Tests: `tests/test_agent_revision_core.rs`. r[lattice.agent_revisions.core] r[lattice.agent_revisions.facts] r[lattice.agent_revisions.framing] r[lattice.agent_revisions.bundle] r[lattice.agent_revisions.identities]
- [x] [serial] 1.2 Add desire, preparation tickets and results, promotion and rollback, session admission and close, revocation, `effect_permitted`, retention holds, cleanup planning and the slots projection. Tests: `tests/test_agent_revision_core.rs`. r[lattice.agent_revisions.preparation] r[lattice.agent_revisions.promotion] r[lattice.agent_revisions.rollback] r[lattice.agent_revisions.sessions] r[lattice.agent_revisions.revocation] r[lattice.agent_revisions.retention] r[lattice.agent_revisions.projection]
- [x] [serial] 1.3 Review fixes in the core: bounds that count only referenced state (live sessions, forgotten closed sessions and unreferenced revisions, a sliding receipt window), receipts bound to the expected rollout generation, `prepare::reissue` for older pending or unknown attempts, no preparation start for a revoked revision, redelivered `prepared` results recorded once, re-admission that rechecks closure and revocation, and revocation never refused for a referenced target. Tests: `tests/test_agent_revision_core.rs`. r[lattice.agent_revisions.core] r[lattice.agent_revisions.preparation] r[lattice.agent_revisions.promotion] r[lattice.agent_revisions.sessions] r[lattice.agent_revisions.revocation]

## Phase 2: Shell

- [x] [serial] 2.1 Add the version-checked `agent_revision_lifecycles_v1` table in `src/store/revisions.rs`. Tests: `tests/test_agent_revision_shell.rs`. r[lattice.agent_revisions.persistence]
- [x] [serial] 2.2 Add `src/agent_revision/` with the injected ports, unavailable and fixture implementations, bundle resolution, the `agent-revision` CLI and the session-admission seam. Tests: `tests/test_agent_revision_shell.rs`. r[lattice.agent_revisions.ports] r[lattice.agent_revisions.shell] r[lattice.agent_revisions.seam]
- [x] [serial] 2.3 Review fixes in the shell: a refused `admit-session` exits non-zero, `prepare` falls back to `prepare::reissue`, `agent_revision::transact` is public for embedders, and the seam takes the admission mode its caller records (`managed` or `unmanaged`). Tests: `tests/test_agent_revision_shell.rs`, including the `lattice` binary and an interleaved stale write. r[lattice.agent_revisions.shell] r[lattice.agent_revisions.seam] r[lattice.agent_revisions.persistence] r[lattice.agent_revisions.ports]

## Phase 3: Documentation and evidence

- [x] [serial] 3.1 Write `docs/agent-revisions.md` and the changelog entry, and retain `evidence/fixture-qualification.json` with exact fixture identities and `live_qualification: false`. r[lattice.agent_revisions.identities]
- [x] [serial] 3.2 Document the bounds, reissue, receipt binding and the seam contract with its admission mode in `docs/agent-revisions.md` and the changelog, and compare the fixture identities with the committed evidence in `fixture_identities_are_stable_and_recorded`. r[lattice.agent_revisions.seam] r[lattice.agent_revisions.identities]

## Phase 4: Verification

- [ ] [serial] 4.1 Run the focused tests, clippy `-D warnings`, fmt, `cairn validate` and the change gates, `scripts/check.rs` and `octet-precommit-check`, and record the results in `evidence/fixture-qualification.json`. r[lattice.agent_revisions.core]

Only checked tasks are complete.
