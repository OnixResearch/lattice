## Why

Lattice can persist run records and content references, but its workflow data remains centered on one run and immediate graph edges. Operators and later operations cannot safely ask for one exact prior resource, inspect its lineage, or select bounded historical facts without reopening unrestricted output.

Swamp demonstrates the value of immutable versioned operation outputs and indirect composition through data. Lattice will adapt that product pattern through Preserves-compatible typed resources, opaque file records, BLAKE3 identities, a closed Rust query IR, explicit retention, and strict secret exclusion.

This change follows `introduce-typed-operation-contracts`. It does not make Valence an operational store and does not adopt CEL.

## What Changes

- Add separate immutable `ResourceRecord` and `FileRecord` contracts with exact producer, schema, content, storage, sensitivity, and non-claim facts. r[lattice.operation_data.records]
- Append a new immutable version for each admitted operation output and retain exact-version retrieval beside bounded latest-version selection. r[lattice.operation_data.versions]
- Add a closed pure query IR for exact identity, producer, type, schema, tag, run, workflow revision, and outcome selection. r[lattice.operation_data.query]
- Add explicit ephemeral, run, duration, and retained lifetime classes with safe garbage collection and separately confirmed orphan pruning. r[lattice.operation_data.retention]
- Exclude plaintext secrets, credentials, secret-derived values, private paths, and undeclared sensitive payloads from the operation-data plane. r[lattice.operation_data.confidentiality]
- Add optional `RecordedOnly` Valence projection for selected record metadata without transferring operational storage or truth authority. r[lattice.operation_data.valence_projection]

## Success Criteria

- Equivalent admitted record facts produce identical BLAKE3 identities independent of traversal order or display aliases.
- A new output version never overwrites an older version.
- `latest` resolves to one exact version during planning and records that version in the invocation plan.
- Query planning and reduction are deterministic, bounded, side-effect-free, and unable to execute user code.
- Garbage collection enforces declared policy. Orphan pruning remains preview-first because a branch or catalog change can make ownership temporarily absent.
- Plaintext credential material never reaches resource payloads, file payloads, indexes, tags, receipts, logs, or query output.
- Valence projection records identity and lineage only and never becomes the source for later operation execution.

## False Completion

A hash-only `ArtifactReference` without producer lineage, schema identity, version semantics, retention, and bounded retrieval is not completion.

An arbitrary JSON predicate, embedded CEL evaluator, SQL pass-through, or report function with ambient store access is not the closed query boundary.

Redacting a stored secret only during display is not secret exclusion.

## Impact

- **Lattice core**: pure record validation, canonical identity, version selection, query reduction, retention planning, pruning planning, and Valence projection.
- **Lattice shell**: capability-confined payload storage, redb indexes, atomic append, bounded retrieval, garbage collection, preview-first prune, and optional projection output.
- **Valence**: consumes selected saved projection artifacts only through existing Evidence IR roles. No Valence source change is required for the first slice.
- **Operation contracts**: output contracts declare resource or file records and provide the producer identity source.
- **Testing**: positive and negative persistence, query, concurrency, retention, redaction, crash, stale-reference, and projection cases.

## Out of Scope

- A distributed datastore, shared S3 backend, remote synchronization, full-text search, SQL API, or arbitrary query language.
- Secret storage, secret-derived downstream data, opaque encryption claims, or regulated-data retention guarantees.
- Registry publication, operation-pack installation, cross-tenant access control, or external data truth.
- Claims for storage durability under arbitrary host failure, remote completeness, semantic correctness, or release eligibility.

## Dependencies

- Local Cairn change: `introduce-typed-operation-contracts`.

## Affected Specs

- `operation-data`: records, versions, query, retention, confidentiality, and Valence projection.
