# Immutable operation data

Lattice stores selected cross-run operation outputs as immutable resource or file versions.

A resource contains canonical structured JSON, one exact schema identity, and an ordered set of declared selectable paths. A file contains opaque bytes, a media class, and an exact content identity.

Each version binds its operation type, type revision, instance, method, invocation, run, node, workflow revision, output contract, tags, schema, content, storage locator, sensitivity, lifetime, retention policy, outcome, and non-claims.

## Query boundary

Nickel can author selectors through `policy/operation-data.ncl`. Rust admits the closed selector labels and all cross-field rules.

The query core supports exact metadata predicates, stable version order, declared resource field paths, and a bounded result limit. It does not support SQL, CEL, regular expressions, scripts, closures, network access, file access, or arbitrary payload code.

Metadata reduction occurs before selected payload reads. Unrelated records cannot cause ambient payload reads. File payloads remain opaque.

A successful dynamic selection becomes an exact version and content identity in a `BoundInvocationPlan` before dispatch. The outer identity binds these exact inputs while preserving the same inner `InvocationPlan` for direct and promoted execution. A final recheck rejects changed inputs.

## Storage and recovery

redb stores immutable metadata and bounded indexes. A capability directory stores content-addressed payloads.

Publication writes and synchronizes a staging file before create-only payload publication. It commits metadata and indexes only after payload verification.

A capability-relative mutation-owner file serializes publication and deletion. An active or crash-left owner blocks new mutation. Recovery reports that state as unknown until an operator repairs it.

Equivalent publication is idempotent. The same version identity with different metadata or bytes fails closed.

Recovery reports missing indexed payloads as incomplete. It reports unindexed payloads as unknown. It does not invent an admitted version.

## Retention

Lifetime classes are ephemeral, run, duration, and retained. Version-count limits can also select old versions.

Garbage collection uses only declared lifetime and version policy. It never deletes a record only because an operation type or instance is absent.

Orphan pruning is a separate preview-first command. Use `--confirm-delete` only after review.

Deletion receipts do not claim secure erasure or recovery impossibility.

## Confidentiality

Admission rejects secret-bearing field names, authorization values, configured prohibited fragments, private host paths, and undeclared sensitive data before publication.

Query output and receipts contain identities and bounded metadata. They do not contain payload bytes.

Explicitly admitted sensitive records still require separate query or export authority.

## Commands

```text
lattice operation-data list
lattice operation-data show <version-b3>
lattice operation-data get <version-b3> <destination> [--allow-sensitive]
lattice operation-data query --selector <query.ncl> [--allow-sensitive]
lattice operation-data garbage-collect --now-epoch-seconds <epoch>
lattice operation-data garbage-collect --now-epoch-seconds <epoch> --confirm-delete
lattice operation-data orphan-prune --catalog <catalog.ncl> --instances <instances.ncl>
lattice operation-data project <version-b3> <destination>
lattice operation-data project <version-b3> <destination> --confirm-create
lattice operation-data recover
```

List, show, query, previews, and projection previews do not delete data or create output files. Exact get and projection use create-only capability-relative publication.

## Valence boundary

Projection emits validated non-sensitive metadata with the `RecordedOnly` role. It omits operational payload by default. The publication shell revalidates the role, non-claims, and projection identity before create-only output.

Valence does not become an operational store. A projection does not make a record executable, true, current, authorized, retained, or release-eligible.

## Non-claims

Operation data does not prove payload truth, external-system state, semantic schema correctness, secure erasure, arbitrary crash durability, distributed consistency, remote completeness, access-control completeness, production readiness, or release eligibility.
