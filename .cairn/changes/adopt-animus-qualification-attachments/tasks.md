## Phase 1: Producer snapshot

- [x] [serial] 1.1 Set `EXPECTED_REVISION` and `SOURCE_ARCHIVE_BLAKE3` in `scripts/sync_animus.rs` for Animus `ed4db4ae00cfec2c2d39bd4a475205ac369f28e8`, run the script into `vendor-cargo-deps/animus`, and record the diff summary in `evidence/producer-adoption.json`. r[lattice.agent_revisions.attachments.vendor]
- [x] [serial] 1.2 Re-sync to Animus `e6f27948b19ea2e1bcb67428ca2e12f7acb6e558` (archive BLAKE3 `6a231c9738f9634df311b58a97e27bdc16e77eced28d4f73327671dafec7ef10`), whose qualification codec changes satisfy workspace Octet without changing behavior, and record the sync history `1c370d50` → `8d6f67d3` → `ed4db4ae` → `e6f27948` in `evidence/producer-adoption.json`. r[lattice.agent_revisions.attachments.vendor]

## Phase 2: Port and CLI

- [x] [serial] 2.1 Add `AttachmentQualification` in `src/agent_revision/port.rs`: bounded regular-file intake, `qualify` over the exact query and exhaustive answer mapping. Tests: `tests/test_agent_revision_attachments.rs`. r[lattice.agent_revisions.attachments]
- [x] [serial] 2.2 Add the repeatable `--qualification-attachment` to promote and rollback and select the attachment port in `src/agent_revision/commands.rs`. Tests: `tests/test_agent_revision_attachments.rs`. r[lattice.agent_revisions.attachments.cli]
- [x] [serial] 2.3 Add `ports::Subject` to `Attachment`, require canonical subject spellings, refuse a qualified answer for another subject in `accept`, and fill the subject from the decoded attachment in the port. Tests: `tests/test_agent_revision_attachments.rs`. r[lattice.agent_revisions.attachments.subject]

## Phase 3: Documentation

- [x] [serial] 3.1 Document the operator procedure, the answer mapping, the owner and the non-claims in `docs/agent-revisions.md`, and add the changelog entry. r[lattice.agent_revisions.attachments]

## Phase 4: Verification

- [x] [serial] 4.1 Run the focused tests, `cargo test -p lattice --lib -- agent_revision`, the agent revision integration suites, clippy with `scripts/check.rs`'s arguments, `cargo fmt --check`, `cairn validate` and the change gates and `octet-precommit-check` at Animus `e6f27948`, and record the results in `evidence/producer-adoption.json` (`commands_final`). r[lattice.agent_revisions.attachments]

Only checked tasks are complete.
