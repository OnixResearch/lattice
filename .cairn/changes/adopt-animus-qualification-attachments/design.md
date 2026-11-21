# Design: Adopt Animus qualification attachments

## Context

Baseline: Lattice `f2d5d8bd` (GitHub `main` `9a74e7ef` plus the lifecycle closeout and the `manage-agent-revisions` core).

- `QualificationPort::answer(&Query) -> Verdict` (`src/agent_revision/mod.rs`) receives the definition identity, the bare revision, the task family, the required class and the shell's `now_ms`. `ops::selection::accept` already refuses an answer that is not canonical, has another class or names another `qualification_ref`.
- Animus `e6f27948b19ea2e1bcb67428ca2e12f7acb6e558` exports `animus_runtime::qualification` under every feature set, including Lattice's `default-features = false, features = ["steel"]`. It needs only `blake3`, which the vendored crate already uses. Its `QualificationAnswer::Qualified` carries the attachment's `provenance_label`, and its conflict detection also considers stale or unusable members.

## Decision 1: re-vendor through the sync script

`EXPECTED_REVISION` and `SOURCE_ARCHIVE_BLAKE3` move to the new revision. The BLAKE3 covers `git -c tar.umask=0000 archive --format=tar` at that revision. The method reproduces the previous value `ff16f090…` for `1c370d50…`. The sync changes exactly the Animus files that differ between the two revisions inside the vendored selection, plus `source.ncl`. The sync history is `1c370d50` → `8d6f67d3` (first adoption) → `ed4db4ae` (review fixes; five `qualification/` files and `source.ncl`) → `e6f27948` (workspace-Octet fixes in `qualification/{codec,consume,json,tests}.rs` and `source.ncl`, with no behavior change; the producer commit's two other changed files are Animus lifecycle files outside the vendored selection). `ed4db4ae` and `e6f27948` are the published forms of two earlier unpublished Animus commits, rewritten only to replace a test literal in `qualification/tests.rs` that secret scanning flagged as a live payment key.

## Decision 2: one port over `qualify`

`AttachmentQualification` holds the bytes of up to 16 files and answers with `qualify(&attachments, &ExpectedSubject { … })`:

| `ExpectedSubject` field | Value |
|---|---|
| `lattice_definition_identity` | `Query::definition_identity` (`blake3:<hex>`) |
| `lattice_profile_revision` | `Query::profile_revision` (bare hex) |
| `task_family` | `Query::task_family` |
| `required_class` | `Query::required_class` (the promotion policy), mapped one to one |
| `now_ms` | `Query::now_ms` (the shell clock) |
| `required_components`, `current_components` | empty |

The answer maps with exhaustive matches and no catch-all:

| Animus | Lattice `Verdict` |
|---|---|
| `Qualified(q)` | `Qualified(Attachment { qualification_ref, evidence_class, protocol_identity, verifier_identity, observed_at_ms })` |
| `NotQualified(r)` | `NotQualified { reason }`: `malformed`, `subject_mismatch`, `scope_mismatch`, `insufficient_class`, `unusable`, `stale`, `conflicting` reason for reason |
| `Unavailable` | `Unavailable {}` |

An empty attachment set answers `Unavailable` without calling the producer, so the refusal keeps owner `animus#18`. The core's existing `accept` keeps the named-reference equality check (`qualification_mismatch`) for promotion and rollback. `Qualified` maps with the subject of Decision 5, taken from the chosen attachment: the port decodes each supplied attachment with `decode_attachment` and uses the one whose identity equals the answer's `qualification_ref`. If none matches, which `qualify` never produces, the answer is `malformed`.

Rejected: parsing attachment JSON in Lattice. The producer owns the canonical form, the redaction rules and the stage order, and a second parser would drift.

## Decision 3: bounded file intake

The shell reads each named path with `symlink_metadata` first and refuses anything that is not a regular file, including a symlink. It then opens the file with `O_NOFOLLOW | O_NONBLOCK`, checks the opened file's metadata again and reads at most 16,384 bytes (`MAX_ATTACHMENT_BYTES`) plus one probe byte. A missing, non-regular or oversized file fails the command before any query, with nothing written. One trailing newline is dropped because the producer's checked files are written as canonical bytes plus a newline. The identity still covers only the canonical bytes, and any other difference is `malformed`.

## Decision 4: CLI selection

`SelectArgs` gains `qualification_attachments: Vec<PathBuf>` (`--qualification-attachment`, repeatable). `commands::execute` uses `AttachmentQualification` for promote and rollback when the list is non-empty and the injected port otherwise. Named files therefore always take effect, and an embedder's port answers when none are named. The standalone CLI keeps `UnavailableQualification` installed, so a command without files is still `qualification_unavailable`.

## Decision 5: answers carry and are checked against their subject

Review finding 7 on the #25 core: `accept` checked the class and the named reference but never the subject, so a faulty port could return another subject's attachment that names the requested reference, and promotion would accept it. `ports::Attachment` now carries `subject: Subject { definition_identity, profile_revision, task_family }`. `is_canonical` also requires a `blake3:` definition identity, 64 lowercase hex for the revision and the capability name rule for the task family. After the class and reference checks, `accept` refuses a subject with another definition or revision as `not_qualified` / `subject_mismatch`, and another task family as `not_qualified` / `scope_mismatch`. These reuse the producer's reasons, which mean exactly this, so the closed refusal vocabulary is unchanged. `accept` takes the lifecycle's definition; its one call site in `select` passes it.

Rejected: copying the subject from the query in the port. That would make the check vacuous for exactly the faulty-port case it guards.

## Tests

`tests/test_agent_revision_attachments.rs` encodes attachments with Animus's `encode_attachment` for the subject Lattice resolves from a request, and drives the standalone CLI shell (`run_command`) and the `lattice` binary:

- a qualified promotion through the binary records `qualification_ref` equal to the attachment identity, and repeats as `already_applied`;
- revoking the reference refuses new sessions and blocks the pinned session;
- rollback with the earlier revision's attachment selects it, while no attachment or another revision's attachment is refused;
- `subject_mismatch` (wrong revision, wrong definition), `scope_mismatch`, `insufficient_class`, `stale`, `unusable`, `conflicting` and `malformed` each refuse with that reason and leave the lifecycle unchanged;
- a named reference different from the answer is `qualification_mismatch`;
- oversized, symlinked, missing and directory paths fail the command;
- no attachment stays `qualification_unavailable` with owner `animus#18`;
- through an injected port, a well-formed `qualified` answer for the exact subject promotes and records the subject, while one for another definition, revision or task family that names the requested reference is refused (`subject_mismatch`, `subject_mismatch`, `scope_mismatch`) and leaves the lifecycle unchanged;
- the vendored snapshot metadata, the sync script and the manifest name the adopted revision.

The existing `test_agent_revision_core.rs` and `test_agent_revision_shell.rs` fixtures give their answers the subject of the revision they promote.

## Risks and claim boundaries

- Evidence is fixture-only. No live producer has qualified a Lattice revision, and a qualified answer claims no live efficacy.
- A favorable attachment is not authority. Promotion and rollback stay explicit commands, and revocation still applies.
- Without `current_components`, the port cannot detect drift in components outside the profile revision. The revision itself is an exact key.
- The Animus revision is a local, unpublished commit. A producer follow-up requires a re-sync and a new archive BLAKE3 before publication.
