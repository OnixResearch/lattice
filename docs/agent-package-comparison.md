# Agent package comparison: decision

Decision: **useful mapping, adoption deferred.** The draft OCI agent-package
envelope maps cleanly onto Lattice bundle identities and the existing revision
lifecycle at the identity level. Lattice adopts no new distribution format,
registry, installer or activation controller. This answers the "NVIDIA
research follow-up" of [Lattice #25](https://github.com/OnixResearch/lattice/issues/25).

This is a **draft-schema fixture comparison, not live OCI interoperability**.
No archive was read, extracted or installed, no registry was contacted, and
no package was promoted.

## Source

- [NVIDIA/agent-package-spec at `697872570744c8cfc1cf6b1d72e04db66e1a2c83`](https://github.com/NVIDIA/agent-package-spec/blob/697872570744c8cfc1cf6b1d72e04db66e1a2c83/spec/agent-package.md),
  a community-review draft, not an adopted standard. No conforming producer
  or consumer was available to exercise.
- Lattice reuses no text or code from the proposal. Its documentation is
  CC-BY-4.0 and its source Apache-2.0; review the
  [pinned license](https://github.com/NVIDIA/agent-package-spec/blob/697872570744c8cfc1cf6b1d72e04db66e1a2c83/LICENSE)
  before any reuse.

## Evidence

| Item | Location |
|---|---|
| Reviewed skill fixture (`case-triage`) | `tests/fixtures/agent-package/skill.json` |
| Plugin with bundled skills (`support-operations`) | `tests/fixtures/agent-package/plugin.json` |
| 34 negative cases with their exact findings | `tests/fixtures/agent-package/negative-cases.json` |
| Comparison report (mappings, identities, decision) | `tests/fixtures/agent-package/comparison-report.json` |
| Harness | `tests/test_agent_package_comparison.rs` |
| Change package | `.cairn/changes/bind-workflow-agent-revisions/` |

Each fixture states the OCI manifest and Package Config as the draft shapes
them, plus an `observation`: the facts a materialization owner would report
about the payload (entry kinds, paths, file bytes with their reported SHA-256,
layer and expanded sizes). The harness compares those supplied facts; it does
not produce them. It recomputes each regular file's SHA-256 from the supplied
bytes and compares that, not the reported digest, with the inventory; a
reported digest that disagrees with the bytes is a finding of its own. The
per-file SHA-256 values in the fixtures are the real digests of their
contents. The manifest, config and layer digests are fixed, well-formed
values that stand for an artifact; no OCI bytes back them.

## Mapping

| Concept | Draft | Lattice / Onix role | Fixture result |
|---|---|---|---|
| Source revision | `org.opencontainers.image.revision` annotation | Opaque producer field, recorded verbatim | Recorded, never an identity input |
| Distribution identity | Manifest digest (`sha256:`) | Provider-native; Mantle's "layer/config/manifest descriptor" role | Recorded in `provider_native`, compared exactly with the observation |
| Config and layer | Config and layer descriptors (`sha256:`) | Provider-native, as in Mantle's `mantle-oci-projection-v1` digest roles | Recorded, compared with the observation |
| Per-file inventory | `files[]`: path, `sha256:` digest, `executable` | `lattice-agent-package-inventory-v1` BLAKE3 over the sorted triples | Derived; the config JSON has no canonical serialization, so its own digest is never recomputed |
| Materialized identity | Materialized tree | `lattice-agent-package-materialized-v1` BLAKE3 over payload-relative path, BLAKE3 of the bytes and `executable`; the analogue of a Mantle object ref | Derived from the supplied bytes only when every check agrees |
| Dependency closure | Self-contained v1; composition by reference is future work | `lattice-agent-package-closure-v1` over no external references and each bundled skill's tree; a tree is `lattice-agent-package-skill-tree-v1` over its skill-relative paths, BLAKE3 of the bytes and `executable` | The plugin closes over `case-triage` and `customer-summary`; the bundled `case-triage` tree (`SKILL.md`, `references/policy.md`, `scripts/analyze.py`) has the reviewed skill's exact tree identity, and dropping `references/policy.md` is `missing_file` and a different tree |
| Evidence subject | Referrer `subject` = manifest digest | `lattice-agent-package-subject-v1` over artifact type, payload id and spec, manifest digest, inventory, materialized and closure identities | One subject per package; the plugin's subject differs from its bundled skill's |
| Lattice bundle | — | A lifecycle `Bundle` whose revision is the evidence subject | Desired, prepared, qualified, promoted, pinned, revoked and rolled back through the existing lifecycle |

The `sha256:` digests never become canonical identities, and the canonical
BLAKE3 identities never stand in for OCI digests. Registry, repository and
tag are excluded from every identity. `org.opencontainers.image.version` is
not a direct input to any identity, but it is part of the manifest bytes:
changing it changes the manifest digest, which the evidence subject binds, so
the lifecycle revision changes too. The harness shows both halves: changing
only the annotation under the same declared digest keeps the revision, and
the new manifest digest changes it.

## Comparison findings

Each negative case mutates one fixture and must report exactly its expected
findings, from a closed vocabulary:

| Class | Findings | Cases |
|---|---|---|
| Inventory and payload | `missing_file`, `extra_file`, `changed_file`, `reported_digest_mismatch`, `duplicate_entry`, `duplicate_inventory_path`, `path_collision` (case folding), `executable_mismatch` | missing, extra, changed, changed bundled skill, content-only change (bytes changed, reported digest not), incomplete bundled skill, duplicate entry, duplicate inventory path, case-folding collision, executable mode |
| Unsafe paths | `unsafe_path` | parent traversal, absolute, empty component, `.` component, backslash, drive prefix, outside `payload/`, NUL byte |
| Entry types | `symlink`, `hard_link`, `special_file` | symlink, hard link, FIFO, character device, block device, socket |
| Decompression bounds | `entry_count_exceeded`, `expanded_bytes_exceeded`, `expansion_ratio_exceeded`, `layer_size_mismatch` | 64 entries, 64 KiB expanded, 64× expansion (fixture bounds; the draft sets none); an observed layer size that differs from the descriptor's is a finding, and the ratio is then taken over the smaller size |
| Manifest and inventory | `kind_mismatch`, `layer_media_type_mismatch`, `layer_count`, `manifest_digest_mismatch`, `config_digest_mismatch`, `layer_digest_mismatch`, `unsupported_envelope` | artifact type and kind, layer media type, second layer, layer, config and manifest digests |

Agreement means only that the supplied facts are consistent. It does not
prove safe extraction or exact installed bytes; that remains with the actual
materialization owner.

## Lifecycle cases

`payload_change_relocation_revocation_and_rollback_use_the_existing_lifecycle`
runs the existing `lattice_core::agent_revision` lifecycle with the package's
evidence subject as the revision:

- **Payload change after review.** The `case-triage` skill is republished with
  a changed `SKILL.md`, a new manifest digest and the same name and version
  annotation. Its revision differs. Promoting it with the reviewed
  revision's qualification reference is refused (`qualification_mismatch`),
  and an evaluation of the reviewed revision under a fresh reference is
  refused (`not_qualified` / `subject_mismatch`).
- **Relocation.** The same package at another registry, repository and tag
  keeps its revision; desiring it again is `unchanged`.
- **Revocation.** After the changed revision is promoted and pinned, revoking
  its digest blocks its session's effects and refuses new admission
  (`revision_revoked`). Desiring or rolling back to a revoked digest is
  refused.
- **Rollback.** Rolling back to the reviewed revision serves new sessions,
  while the session pinned to the revoked digest stays blocked: a historical
  pin never restores revoked permissions.

## Why adoption is deferred

- No conforming producer or consumer exists, so only the draft's shapes could
  be compared. The draft leaves archive limits (open question 5) and the
  unknown-field and extension policy (open question 7) open; the harness fails
  closed on non-empty `extensions`.
- The draft forbids symlinks. Mantle's `mantle-canonical-archive-v1` admits
  relative non-escaping symlinks, so an agent-package layer is a stricter
  profile that Mantle would have to select explicitly. Mantle imports an
  external layout only as `compatibility-only`, never with an Onix identity.
- Per-file digests are SHA-256 only. Canonical identities need the bytes,
  which only a materialization owner observes.
- Signatures, scans, behavioral qualification, approval and activation stay
  separate. Animus #18 owns qualified attachments and Kiln keeps candidate
  acceptance; a referrer would have to be bound to the evidence subject and a
  supported schema before it could become an attachment.

Revisit when a conforming producer exists and a materialization owner can
report the observation facts for an actual layer.

## Non-claims

- No live OCI interoperability, registry access, signature verification,
  scanning, installation or activation.
- No safe-extraction claim: the harness never reads an archive.
- The fixtures are synthetic and the decompression bounds are fixture
  choices.
