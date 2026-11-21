## Phase 1: Pure core

- [x] [serial] 1.1 Add `crates/lattice-core/src/agent_revision/` with the bounded lifecycle state, closed refusals, bundle facts, the components framing extension, lifecycle identities and port facts. Tests: `tests/test_agent_revision_core.rs`. r[lattice.agent_revisions.core] r[lattice.agent_revisions.facts] r[lattice.agent_revisions.framing] r[lattice.agent_revisions.bundle] r[lattice.agent_revisions.identities]
- [x] [serial] 1.2 Add desire, preparation tickets and results, promotion and rollback, session admission and close, revocation, `effect_permitted`, retention holds, cleanup planning and the slots projection. Tests: `tests/test_agent_revision_core.rs`. r[lattice.agent_revisions.preparation] r[lattice.agent_revisions.promotion] r[lattice.agent_revisions.rollback] r[lattice.agent_revisions.sessions] r[lattice.agent_revisions.revocation] r[lattice.agent_revisions.retention] r[lattice.agent_revisions.projection]

## Phase 2: Shell

- [x] [serial] 2.1 Add the version-checked `agent_revision_lifecycles_v1` table in `src/store/revisions.rs`. Tests: `tests/test_agent_revision_shell.rs`. r[lattice.agent_revisions.persistence]
- [x] [serial] 2.2 Add `src/agent_revision/` with the injected ports, unavailable and fixture implementations, bundle resolution, the `agent-revision` CLI and the session-admission seam. Tests: `tests/test_agent_revision_shell.rs`. r[lattice.agent_revisions.ports] r[lattice.agent_revisions.shell] r[lattice.agent_revisions.seam]

## Phase 3: Documentation and evidence

- [x] [serial] 3.1 Write `docs/agent-revisions.md` and the changelog entry, and retain `evidence/fixture-qualification.json` with exact fixture identities and `live_qualification: false`. r[lattice.agent_revisions.identities]

## Phase 4: Verification

- [ ] [serial] 4.1 Run the focused tests, clippy `-D warnings`, fmt, `cairn validate` and the change gates, `scripts/check.rs` and `octet-precommit-check`, and record the results in `evidence/fixture-qualification.json`. r[lattice.agent_revisions.core]

Only checked tasks are complete.
