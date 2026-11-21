# Manage immutable agent revisions

Tracking: [Lattice #25](https://github.com/OnixResearch/lattice/issues/25), sections 1 to 5, and the integration of its session seam into `implementation-attempt`. Section 6 (inventory publication) is `publish-worker-inventory-v3`; this change provides its pure projection. Shared contract with [Animus #18](https://github.com/OnixResearch/animus/issues/18) (qualification attachments). Owner-approved on 2026-11-20.

## Why

A worker profile revision identifies what an implementation attempt runs, but nothing decides which revision new sessions use. Operators cannot prepare a new prompt, tool list, harness or model requirement, qualify it, and promote it explicitly while existing sessions keep the configuration they were admitted under. A failed preparation must not replace the last usable revision, a late result must not overwrite newer state, and pinning must not keep privileges that have since been revoked. `lattice.worker-inventory.v2` publishes the desired, prepared, qualified, selected and pinned slots as unknowns owned by `lattice#25`.

## What Changes

- The lifecycle's revision is the profile revision, `Request::worker_profile_identity()`, used verbatim as an opaque bare 64-hex value. The definition is the inventory's `lattice-worker-definition-v1` identity. r[lattice.agent_revisions.bundle]
- The profile revision frames the reviewed harness template: the OMP launch prefix and print options in argv order and the request schema, read from the constants the shell passes to the worker. Changing a reviewed flag (tools, mode, `--no-session`, approval mode) is a new revision; the task prompt, base snapshot and broker expiry are per-attempt and do not change it. The operator-declared `lattice.agent-revision-components.v1` extension is removed: nothing wired it, and a declaration could disagree with the effective constants. r[lattice.agent_revisions.framing]
- A pure core, `lattice_core::agent_revision`, keeps per definition the desired revision and change generation, generation-bound preparation, qualification per task-family scope, the last usable revision, the selection for new sessions and each session's pin. It implements desire, preparation tickets and results, promotion, rollback, revocation, session admission and close, `effect_permitted`, retention holds and cleanup planning, with a closed refusal vocabulary. r[lattice.agent_revisions.core] r[lattice.agent_revisions.facts] r[lattice.agent_revisions.preparation] r[lattice.agent_revisions.promotion] r[lattice.agent_revisions.rollback] r[lattice.agent_revisions.sessions] r[lattice.agent_revisions.revocation] r[lattice.agent_revisions.retention]
- Preparation, policy, promotion and session pin identities use new `content_identity_core` domains. r[lattice.agent_revisions.identities]
- The store gains one version-checked table, `agent_revision_lifecycles_v1`, in the existing redb database. r[lattice.agent_revisions.persistence]
- The shell injects `PreparationPort` (Kiln #49) and `QualificationPort` (Animus #18). The standalone CLI installs unavailable ports, so preparation records `unknown` and promotion is refused as `qualification_unavailable`. r[lattice.agent_revisions.ports]
- `lattice agent-revision` adds `resolve`, `desire`, `prepare`, `promote`, `rollback`, `revoke`, `admit-session`, `cleanup` and `inspect`, each printing one JSON document. r[lattice.agent_revisions.shell]
- `agent_revision::seam` exposes session admission, the effect check and close for lifecycle-managed definitions; unmanaged definitions answer `unmanaged`. The caller records each session's admission mode (`managed` or `unmanaged`) at admission and passes it back, so sessions admitted before a definition became managed stay unmanaged and a managed session unknown to the lifecycle fails closed. r[lattice.agent_revisions.seam]
- `implementation-attempt run`, `recover`, `export-candidate` and `inspect` wire the seam through an optional `--revision-store`: admission before any write, the admission mode and pin in `intent.json`, the effect check right before spawn, close after the terminal result (`unknown_session` counts as closed), and a refused export once the session, its pinned revision or its qualification is revoked (`sessions::retained_permitted`). Without a store an attempt is unmanaged and unchanged. r[lattice.agent_revisions.seam] r[lattice.agent_revisions.revocation]
- `projection::slots` projects the bounded slots, with qualification provenance, that inventory v3 publishes. r[lattice.agent_revisions.projection]

## Impact

- **Pure core:** new `crates/lattice-core/src/agent_revision/` and one module line in `crates/lattice-core/src/lib.rs`; the harness template and request schema in `content_identity/attempt.rs`.
- **Shell:** new `src/agent_revision/`, new `src/store/revisions.rs` with one `mod` line in `src/store/mod.rs`, and registration lines in `src/lib.rs` and `src/main.rs`; `src/implementation_attempt/cli.rs` (`--revision-store`); new `src/implementation_attempt/cli/session.rs` with `session/{intent,revision,fencing,profile,launch}.rs` (`HarnessTemplate` and `EFFECTIVE_TEMPLATE` in `session/profile.rs`); the run, recovery, refusal and export modules; `src/candidate_worker/omp/profile.rs` exposes its launch prefix and print options as a `pub(crate)` module.
- **Tests:** `tests/test_agent_revision_core.rs`, `tests/test_agent_revision_shell.rs`, `src/implementation_attempt/cli/tests.rs`, `src/implementation_attempt/cli/tests/revisions.rs`.
- **Docs:** new `docs/agent-revisions.md`; changelog.
- **Compatibility:** every profile revision value changes with the harness template framing. Attempts run without `--revision-store` keep their behavior and intent bytes; managed attempts are refused, blocked or not exported as above. Every existing table is unchanged. No Cargo dependency is added.

## Dependencies

- Local: `publish-worker-inventory-v2` (definition identity, capability names), `execute-admitted-implementation-attempt` (the attempt CLI, its intent and recovery).
- Producers: Animus #18 (qualification attachments), Kiln #49 (runtime preparation and cleanup). Neither is installed; the standalone CLI reports them unavailable.

## Out of Scope

- Inventory publication of the slots (section 6, `publish-worker-inventory-v3`) and Kiln #60 presentation.
- Live preparation, qualification, canaries or rollout; this change is fixture-qualified only.
- Checkpoint and resume (Lattice #12); this lifecycle does not make sessions migratable.
- The OCI package comparison and Cairn campaign pinning follow-ups recorded on the issue.
