# Agent Revisions Specification

## ADDED Requirements

### Requirement: The vendored producer is the exact Animus #18 revision
r[lattice.agent_revisions.attachments.vendor] The vendored Animus snapshot MUST come from `scripts/sync_animus.rs` at revision `8d6f67d38509899402df4d62e2d54321ff047f06`, and `source.ncl` MUST record that revision and the BLAKE3 of `git -c tar.umask=0000 archive --format=tar` at it. The `animus-runtime` dependency MUST stay `default-features = false, features = ["steel"]`, and no crate or Cargo dependency MUST be added.

#### Scenario: The snapshot records the adopted revision
r[lattice.agent_revisions.attachments.vendor.scenario.recorded]
- GIVEN the synchronized snapshot
- WHEN its metadata, the sync script and the root manifest are read
- THEN `source.ncl` and the script MUST name the revision and archive BLAKE3, and the `animus-runtime` dependency MUST keep its features

### Requirement: An attachment-backed qualification port
r[lattice.agent_revisions.attachments] `AttachmentQualification` MUST answer the qualification port only through `animus_runtime::qualification::qualify`, with the query's exact definition identity, bare profile revision, task family, required class and `now_ms`. `Qualified` MUST keep `qualification_ref`, evidence class, protocol identity, verifier identity, `observed_at_ms` and the producer's provenance label (`fixture` or `live`). The provenance label MUST be recorded with the admitted qualification and MUST NOT be a promotion condition. Each `NotQualified` reason MUST map to the Lattice reason of the same spelling through exhaustive matches without a catch-all. With no attachment the answer MUST be `unavailable`. The port MUST read each attachment file as at most 16,384 bytes, MUST refuse a missing file, a symlink or any other non-regular file, and MAY drop exactly one trailing newline. A favorable answer MUST NOT select a revision by itself.

#### Scenario: A qualified attachment promotes the exact subject
r[lattice.agent_revisions.attachments.scenario.qualified]
- GIVEN a prepared revision and an attachment for its definition, revision, task family and required class that is fresh at the shell clock
- WHEN `promote` names the attachment's identity and the attachment file
- THEN the revision MUST be selected, the recorded `qualification_ref` MUST equal the attachment identity, and repeating the request MUST be `already_applied`

#### Scenario: Every producer reason refuses promotion
r[lattice.agent_revisions.attachments.scenario.reasons]
- GIVEN attachments for another revision or definition, another task family, another class than required, an expired window, an observation after the shell clock, two distinct attachments from one source, or bytes that are not canonical
- WHEN `promote` names them
- THEN it MUST be refused as `not_qualified` with `subject_mismatch`, `scope_mismatch`, `insufficient_class`, `stale`, `unusable`, `conflicting` or `malformed` respectively, and the lifecycle MUST stay unchanged

#### Scenario: The named reference must equal the answer
r[lattice.agent_revisions.attachments.scenario.named_reference]
- GIVEN a qualifying attachment
- WHEN `promote` names a different `qualification_ref`
- THEN it MUST be refused as `qualification_mismatch`

#### Scenario: Unbounded or indirect files are refused
r[lattice.agent_revisions.attachments.scenario.files]
- GIVEN a file over 16,384 bytes, a symlink to a valid attachment, a missing path or a directory
- WHEN `promote` names it
- THEN the command MUST fail before any query and nothing MUST be written

#### Scenario: Revocation still applies after a qualified promotion
r[lattice.agent_revisions.attachments.scenario.revoked]
- GIVEN a revision promoted with an attachment and a session pinned to it
- WHEN the attachment's `qualification_ref` is revoked
- THEN a new session MUST be refused as `qualification_revoked` and the pinned session's effects MUST be blocked

#### Scenario: The provenance label is recorded, not gated
r[lattice.agent_revisions.attachments.scenario.provenance]
- GIVEN a qualifying attachment labelled `live`
- WHEN `promote` names it
- THEN the revision MUST be selected and the admitted qualification MUST record provenance `live`

### Requirement: A qualified answer is bound to the queried subject
r[lattice.agent_revisions.attachments.subject] A `qualified` answer MUST carry the subject it was produced for: a `blake3:` definition identity, a bare 64 lowercase hex profile revision and a capability-name task family, and an answer with any other spelling MUST be `malformed`. The attachment port MUST take the subject from the attachment Animus decoded, never from the question. Promotion and rollback MUST be refused as `not_qualified` with `subject_mismatch` when the subject names another definition or revision, and with `scope_mismatch` when it names another task family, even when the answer names the requested `qualification_ref`.

#### Scenario: The exact subject is accepted and recorded
r[lattice.agent_revisions.attachments.subject.scenario.bound]
- GIVEN an injected port that answers `qualified` for the exact definition, revision and task family
- WHEN `promote` names the answer's reference
- THEN the revision MUST be selected and the recorded attachment MUST carry that subject

#### Scenario: Another subject's answer is refused
r[lattice.agent_revisions.attachments.subject.scenario.foreign]
- GIVEN an injected port that answers the exact query with a well-formed `qualified` attachment naming the requested reference but another definition, another revision or another task family
- WHEN `promote` runs
- THEN it MUST be refused as `not_qualified` with `subject_mismatch`, `subject_mismatch` or `scope_mismatch` respectively, and the lifecycle MUST stay unchanged

### Requirement: Attachments are named per command
r[lattice.agent_revisions.attachments.cli] `agent-revision promote` and `rollback` MUST accept a repeatable `--qualification-attachment PATH`, at most 16 per command. With one or more files the attachment port MUST answer; without any the injected port MUST answer, so the standalone CLI MUST still refuse as `qualification_unavailable` with owner `animus#18`.

#### Scenario: No attachment stays unavailable
r[lattice.agent_revisions.attachments.scenario.unavailable]
- GIVEN a prepared revision and the standalone CLI
- WHEN `promote` names no attachment file
- THEN it MUST be refused as `qualification_unavailable` with owner `animus#18`

#### Scenario: Rollback requires the earlier revision's attachment
r[lattice.agent_revisions.attachments.scenario.rollback]
- GIVEN R1 promoted with attachment A1, then R2 promoted with attachment A2
- WHEN rollback to R1 names no file, then A2, then A1
- THEN it MUST be refused as `qualification_unavailable`, then as `not_qualified` with `subject_mismatch`, and then R1 MUST be selected with A1's identity
