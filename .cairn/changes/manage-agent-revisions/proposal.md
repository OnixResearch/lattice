# Manage immutable agent revisions

Tracking: [Lattice #25](https://github.com/OnixResearch/lattice/issues/25), sections 1 to 5. Section 6 (inventory publication) is a later step; this change exposes its pure projection only. Shared contract with [Animus #18](https://github.com/OnixResearch/animus/issues/18) (qualification attachments). Owner-approved on 2026-11-20.

## Why

A worker profile revision identifies what an implementation attempt runs, but nothing decides which revision new sessions use. Operators cannot prepare a new prompt, tool list, harness or model requirement, qualify it, and promote it explicitly while existing sessions keep the configuration they were admitted under. A failed preparation must not replace the last usable revision, a late result must not overwrite newer state, and pinning must not keep privileges that have since been revoked. `lattice.worker-inventory.v2` publishes the desired, prepared, qualified, selected and pinned slots as unknowns owned by `lattice#25`.

## What Changes

- The lifecycle's revision is the existing profile revision, `Request::worker_profile_identity()`, used verbatim as an opaque bare 64-hex value. The definition is the inventory's `lattice-worker-definition-v1` identity. r[lattice.agent_revisions.bundle]
- An optional `lattice.agent-revision-components.v1` declaration covers the issue's section 1 components the revision does not frame yet: memory policy, checkpoint semantics, tool executor protocol and names, model requirements and producer/consumer compatibility. Its digest is framed only when declared, so existing revisions keep their bytes. Wiring it into the revision is a one-line change in `src/implementation_attempt/cli.rs`, which this change does not edit. r[lattice.agent_revisions.framing]
- A pure core, `lattice_core::agent_revision`, keeps per definition the desired revision and change generation, generation-bound preparation, qualification per task-family scope, the last usable revision, the selection for new sessions and each session's pin. It implements desire, preparation tickets and results, promotion, rollback, revocation, session admission and close, `effect_permitted`, retention holds and cleanup planning, with a closed refusal vocabulary. r[lattice.agent_revisions.core] r[lattice.agent_revisions.facts] r[lattice.agent_revisions.preparation] r[lattice.agent_revisions.promotion] r[lattice.agent_revisions.rollback] r[lattice.agent_revisions.sessions] r[lattice.agent_revisions.revocation] r[lattice.agent_revisions.retention]
- Preparation, policy, promotion and session pin identities use new `content_identity_core` domains. r[lattice.agent_revisions.identities]
- The store gains one version-checked table, `agent_revision_lifecycles_v1`, in the existing redb database. r[lattice.agent_revisions.persistence]
- The shell injects `PreparationPort` (Kiln #49) and `QualificationPort` (Animus #18). The standalone CLI installs unavailable ports, so preparation records `unknown` and promotion is refused as `qualification_unavailable`. r[lattice.agent_revisions.ports]
- `lattice agent-revision` adds `resolve`, `desire`, `prepare`, `promote`, `rollback`, `revoke`, `admit-session`, `cleanup` and `inspect`, each printing one JSON document. r[lattice.agent_revisions.shell]
- `agent_revision::seam` exposes session admission, the effect check and close for lifecycle-managed definitions; unmanaged definitions answer `unmanaged`. The caller records each session's admission mode (`managed` or `unmanaged`) at admission and passes it back, so sessions admitted before a definition became managed stay unmanaged and a managed session unknown to the lifecycle fails closed. r[lattice.agent_revisions.seam]
- `projection::slots` projects the bounded slots a later inventory schema can publish. r[lattice.agent_revisions.projection]

## Impact

- **Pure core:** new `crates/lattice-core/src/agent_revision/` and one module line in `crates/lattice-core/src/lib.rs`.
- **Shell:** new `src/agent_revision/`, new `src/store/revisions.rs` with one `mod` line in `src/store/mod.rs`, and registration lines in `src/lib.rs` and `src/main.rs`.
- **Tests:** `tests/test_agent_revision_core.rs`, `tests/test_agent_revision_shell.rs`.
- **Docs:** new `docs/agent-revisions.md`; changelog.
- **Compatibility:** additive. Implementation attempts, their profile revisions, the inventory and every existing table are unchanged. No Cargo dependency is added.

## Dependencies

- Local: `publish-worker-inventory-v2` (definition identity, capability names, revision slots).
- Producers: Animus #18 (qualification attachments), Kiln #49 (runtime preparation and cleanup). Neither is installed; the standalone CLI reports them unavailable.
- Integration: the implementation-attempt slice owns the call-site change for the components digest, the stabilized profile revision and wiring of the seam.

## Out of Scope

- Inventory publication of the slots (section 6) and Kiln #60 presentation.
- Live preparation, qualification, canaries or rollout; this change is fixture-qualified only.
- Checkpoint and resume (Lattice #12); this lifecycle does not make sessions migratable.
- The OCI package comparison and Cairn campaign pinning follow-ups recorded on the issue.
