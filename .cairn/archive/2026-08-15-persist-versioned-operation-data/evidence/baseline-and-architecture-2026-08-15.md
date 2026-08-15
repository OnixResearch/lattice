# Operation data baseline and architecture

Date: 2026-08-15
Base: `b80f07ffb5e196944d40758dcb94e83b377012a0`

## Dependency

`introduce-typed-operation-contracts` is archived at `.cairn/archive/2026-08-15-introduce-typed-operation-contracts`. Its accepted specification is `.cairn/specs/operation-contracts/spec.md`.

## Pre-change baseline

These suites passed before core changes:

- `test_operation_contracts`: 15 tests;
- `test_replay`: 10 tests;
- `test_run_inspection`: 16 tests;
- `test_store`: 29 tests;
- `test_workflow`: 23 tests.

Strict Cairn validation and the task gate passed.

## Existing ownership

- `ArtifactReference` is a replay-safe reference. It is not a versioned operational record.
- `ValueStore` deduplicates immediate values by content. It does not preserve operation lineage or versions.
- `operation_runs_v1` stores metadata-only direct-run evidence.
- redb owns durable metadata and indexes.
- capability directories own payload file access.
- operation contracts own type, method, invocation, output-contract, and authority facts.
- run inspection and replay omit arbitrary payload content.

## Architecture

The functional core owns record admission, canonical identity, publication decisions, bounded query reduction, retention plans, exact selection binding, and recorded-only projection.

The imperative shell owns capability-relative payload I/O, redb transactions, crash observations, create-only export, garbage-collection execution, and projection publication.

Structured resources use deterministic canonical JSON bytes. Opaque files remain byte arrays outside the query core.

The implementation adds application-owned storage ports only for operational record metadata and payload capabilities. It does not make Valence an operational port.

## Adversarial portfolio

The review considered mutable latest rows, content-only references, arbitrary JSON predicates, SQL or CEL, Valence lookup, and immutable exact records.

Only immutable exact records with a closed bounded query IR preserve lineage and dispatch reproducibility. Mutable pointers, arbitrary predicates, and evidence-store lookup are rejected.

The audit will try to falsify these properties:

- identical content from different invocations keeps distinct lineage;
- latest becomes an exact version before dispatch;
- secret material cannot enter payloads, metadata, tags, diagnostics, receipts, or query output;
- garbage collection cannot perform orphan pruning;
- same identity with different bytes cannot replace prior data;
- projection cannot grant authority or become runtime truth.

## Non-claims

The baseline and design do not prove payload truth, secure erasure, arbitrary crash durability, distributed consistency, remote completeness, access-control completeness, confidentiality of approved sensitive data, release eligibility, or release readiness.
