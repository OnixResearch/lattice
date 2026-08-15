## Phase 1: Baseline and ownership

- [x] [depends:introduce-typed-operation-contracts] V1 Begin implementation only after the typed operation output-contract and invocation identities are available. r[lattice.operation_data.records]
- [x] [serial] V2 Run and record current artifact-reference, replay, run-inspection, redb, workflow-output, credential-redaction, crash-boundary, and capability-root baselines. r[lattice.operation_data.records] r[lattice.operation_data.confidentiality]
- [x] [serial] V3 Inventory every existing artifact, output, cache, run record, replay record, safe summary, storage table, retention rule, and evidence projection that can overlap the data plane. r[lattice.operation_data.records] r[lattice.operation_data.retention]

## Phase 2: Record contracts and identities

- [x] [serial] R1 Add pure versioned `ResourceRecord` and `FileRecord` DTO validation with closed fields, named bounds, safe locators, sensitivity classes, lifetimes, and non-claims. r[lattice.operation_data.records]
- [x] [serial] R2 Bind operation type, revision, instance, method, invocation, run, node, workflow revision, output contract, schema, content, storage, lifetime, and policy facts into domain-separated BLAKE3 identities. r[lattice.operation_data.records] r[lattice.operation_data.versions]
- [x] [serial] R3 Define the canonical structured resource representation and preserve opaque file bytes outside the query core. r[lattice.operation_data.records]
- [x] [parallel] R4 Add positive and negative record fixtures for canonical equivalence, changed provenance, malformed schemas, unsafe locators, wrong hashes, duplicate conflicts, oversize, and missing non-claims. r[lattice.operation_data.records]

## Phase 3: Append-only publication and retrieval

- [x] [serial] P1 Add a pure publication state plan for payload verification, metadata append, index updates, duplicate idempotence, conflict denial, and uncertain outcomes. r[lattice.operation_data.versions]
- [x] [serial] P2 Add capability-confined payload storage and redb metadata, lineage, version, tag, and schema indexes through a thin shell. r[lattice.operation_data.versions]
- [x] [serial] P3 Preserve distinct producer lineage when identical content appears in different invocations and reject same identity with different bytes. r[lattice.operation_data.versions]
- [x] [parallel] P4 Add concurrent-writer and crash fixtures for pre-payload, post-payload, pre-index, active-owner, restart, and repair boundaries. r[lattice.operation_data.versions]
- [x] [parallel] P5 Add exact version and derived latest retrieval with final identity and payload validation before return. r[lattice.operation_data.versions]

## Phase 4: Closed query and composition

- [x] [serial] Q1 Add typed Nickel selector contracts and a closed Rust query IR for exact identity, class, operation, method, instance, output, schema, tag, producer, workflow revision, outcome, ordering, and limit facts. r[lattice.operation_data.query]
- [x] [serial] Q2 Add pure bounded query reduction over in-memory candidate metadata and named structured resource paths without SQL, CEL, scripts, regex, file reads, or executor calls. r[lattice.operation_data.query]
- [x] [serial] Q3 Resolve latest and selector inputs to exact version identities during operation planning and bind them into invocation and cache identities. r[lattice.operation_data.query]
- [x] [parallel] Q4 Add negative empty, ambiguous, stale, unauthorized, sensitive, oversized, unsupported-field, unsupported-file-payload, and changed-before-dispatch query cases. r[lattice.operation_data.query]

## Phase 5: Retention and confidentiality

- [x] [serial] L1 Add pure lifetime and retention evaluation for ephemeral, run, duration, retained, and version-count policies. r[lattice.operation_data.retention]
- [x] [serial] L2 Add idempotent garbage-collection execution for explicit policy expiry and a separate preview-first orphan-prune command. r[lattice.operation_data.retention]
- [x] [parallel] L3 Add branch-switch, missing-pack, migration, stale-plan, concurrent-owner, deletion-failure, and no-secure-erasure fixtures. r[lattice.operation_data.retention]
- [x] [serial] S1 Add secret and prohibited-data admission before payload publication, metadata indexing, tagging, summaries, diagnostics, or receipts. r[lattice.operation_data.confidentiality]
- [x] [parallel] S2 Add leak tests for credentials, authorization headers, environment values, secret-derived strings, overlapping fragments, private paths, tags, summaries, rejected payloads, and query output. r[lattice.operation_data.confidentiality]

## Phase 6: Valence projection and product surface

- [x] [serial] E1 Add an optional pure projection from selected safe record metadata to a `RecordedOnly` Valence-compatible artifact draft. r[lattice.operation_data.valence_projection]
- [x] [serial] E2 Add a thin saved-artifact shell and exact revalidation without payload embedding, live Valence access, or reverse operational lookup. r[lattice.operation_data.valence_projection]
- [x] [parallel] E3 Add negative role-promotion, payload-embedding, stale-identity, missing-non-claim, and operational-authority projection fixtures. r[lattice.operation_data.valence_projection]
- [x] [serial] E4 Add headless list, show, exact get, query, garbage-collection preview and execute, orphan-prune preview and execute, and optional projection commands with safe text and JSON. r[lattice.operation_data.query] r[lattice.operation_data.retention]
- [x] [serial] E5 Document record classes, identity, query, composition, retention, secret exclusion, crash recovery, Valence boundary, and non-claims. r[lattice.operation_data.records] r[lattice.operation_data.valence_projection]

## Phase 7: Closeout

- [x] [serial] C1 Run focused tests, formatting, Clippy, `scripts/check.rs`, Octet, Cairn validation and gates, crash-boundary operator evidence, and bounded Nix validation. r[lattice.operation_data.records] r[lattice.operation_data.query] r[lattice.operation_data.confidentiality]
- [x] [serial] C2 Review dry-run sync and archive plans, make sure that every requirement reaches the accepted operation-data spec, and archive only after current positive and negative evidence passes. r[lattice.operation_data.retention] r[lattice.operation_data.valence_projection]
