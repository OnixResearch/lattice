# Adopt Animus qualification attachments

Tracking: [Animus #18](https://github.com/OnixResearch/animus/issues/18) (qualification attachments) consumed by [Lattice #25](https://github.com/OnixResearch/lattice/issues/25) (agent revision promotion). Owner-approved on 2026-11-20.

## Why

`manage-agent-revisions` defines the qualification port, but the standalone CLI installs only `UnavailableQualification`, so promotion and rollback of a prepared revision are always refused as `qualification_unavailable` (owner `animus#18`). Animus `e6f27948b19ea2e1bcb67428ca2e12f7acb6e558` publishes the producer side: canonical `animus.qualification-attachment.v1` documents and a pure validator, `animus_runtime::qualification::qualify`, whose answer vocabulary equals Lattice's port vocabulary. Lattice can now qualify a promotion with real producer evidence instead of reporting the producer as missing.

## What Changes

- Re-vendor Animus at `e6f27948b19ea2e1bcb67428ca2e12f7acb6e558` through `scripts/sync_animus.rs`. The crate list and the `animus-runtime` dependency (`default-features = false, features = ["steel"]`) are unchanged. r[lattice.agent_revisions.attachments.vendor]
- Add `AttachmentQualification`, a qualification port over operator-supplied attachment files. It reads bounded regular files and answers through `qualify` for the exact definition, bare revision, task family, required class and shell clock. It maps every producer answer to the Lattice verdict with exhaustive matches. r[lattice.agent_revisions.attachments]
- `agent-revision promote` and `rollback` accept a repeatable `--qualification-attachment PATH`. With files, the attachment port answers. Without them, the injected port answers, so the standalone CLI still refuses as `qualification_unavailable`. r[lattice.agent_revisions.attachments.cli]
- A `qualified` answer carries the subject it was produced for (`ports::Subject`), and the core refuses an answer for another definition, revision or task family even when it names the requested reference. The attachment port fills the subject from the attachment Animus decoded. This closes review finding 7 on the #25 core, where a faulty port could promote with another subject's attachment. r[lattice.agent_revisions.attachments.subject]
- Document the operator procedure, the owner and the non-claims in `docs/agent-revisions.md`, and add a changelog entry.

## Impact

- **Vendor:** `vendor-cargo-deps/animus/` gains `animus-runtime`'s qualification module: three edited Animus files (`README.md`, `eval_feedback/mod.rs`, `lib.rs`) and seven new ones. `source.ncl` and `vendor-cargo-deps/README.md` record the revision. The stray `vendor-cargo-deps/animus/Cargo.lock`, which no sync produces, is removed.
- **Core:** `crates/lattice-core/src/agent_revision/ports.rs` (`Subject`, `Provenance`, `Attachment::{subject, provenance}`, canonical spelling) and `accept` in `ops/selection.rs` (subject check; its call site passes the lifecycle's definition). The provenance label is recorded with the admitted qualification and never gates promotion.
- **Shell:** `src/agent_revision/port.rs` (port), `src/agent_revision/commands.rs` (port selection), `src/agent_revision/mod.rs` (struct and argument).
- **Tests:** new `tests/test_agent_revision_attachments.rs`. `tests/test_agent_revision_shell.rs` sets the new argument to empty and gives its fixture answer a subject. `tests/test_agent_revision_core.rs` gives its fixture answers a subject.
- **Compatibility:** without the new argument every command behaves as before for a port that answers about the queried subject. `Attachment` gains a required `subject` field, so stored lifecycles from the unreleased #25 core that hold qualifications do not decode; no published release carries them. No Cargo dependency is added.

## Dependencies

- Local: `manage-agent-revisions` (port contract, lifecycle, CLI).
- Producer: Animus `e6f27948b19ea2e1bcb67428ca2e12f7acb6e558` (the #18 change with its review fixes and a workspace-Octet fix in the qualification codec that changes no behavior), first adopted at `8d6f67d38509899402df4d62e2d54321ff047f06` and then at `ed4db4ae00cfec2c2d39bd4a475205ac369f28e8`. Both later revisions were rewritten before publication to replace a test literal that secret scanning flagged; behavior is unchanged. Any further producer commit requires a re-sync with a new archive BLAKE3.

## Out of Scope

- Live qualification. The exercised producer lane is fixture evidence only.
- Required and current component checks (`required_components`, `current_components`); the port passes them empty.
- Compatibility decisions for reuse across revisions (`classify_applicability`).
- Wiring the session seam into implementation attempts.
