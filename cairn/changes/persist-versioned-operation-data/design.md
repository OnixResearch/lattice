## Context

Lattice runner events can carry `ArtifactReference` values with content hash, type tag, media type, byte length, backend reference, sensitivity, and summary. Replay can validate those references and report unavailable payloads. Run inspection intentionally omits content.

The missing capability is an explicit operational data plane that preserves typed outputs across runs without weakening the safe run-inspection boundary. This data plane must support later composition while keeping evidence identity, secret handling, and storage authority separate.

## Success Contract

Completion requires immutable append-only records, exact BLAKE3 identities, bounded indexes, deterministic query reduction, explicit lifetime policy, crash-aware publication, secret exclusion, and positive and negative tests through production store adapters.

A generic database wrapper, mutable latest row, display-only redaction, unbounded predicate evaluator, or Valence-backed runtime lookup is false completion.

## Decisions

### Decision: separate structured resources from opaque files

**Choice:** Define two record classes.

`ResourceRecord` carries a versioned schema identity and canonical structured payload suitable for typed field selection. The canonical payload uses a reviewed Preserves-compatible representation or another exact Lattice-owned canonical form.

`FileRecord` carries media class, byte length, BLAKE3 content identity, backend locator, and bounded safe metadata. Its bytes are opaque to the query core.

Both records bind:

- record schema version and record class;
- operation type, type revision, instance, method, invocation, run, node, and workflow revision identities;
- output name and output-contract identity;
- content identity and exact version identity;
- safe tags and sensitivity class;
- lifetime and retention policy identity;
- storage backend class and safe locator identity;
- explicit non-claims.

**Rationale:** Structured composition and artifact retention need different validation and query behavior.

### Decision: append immutable versions

**Choice:** Every admitted output appends one record version. No operation updates payload or provenance in place. Equivalent content from different invocations remains separately attributable even when content hashes match.

A version identity binds record identity, producer invocation, output role, sequence facts, content identity, and policy identity. `latest` is a derived selection over admitted versions, not a mutable authority pointer.

The shell reserves and writes record metadata and payload through a crash-aware protocol. A missing payload, uncertain publication, or index disagreement remains explicit and cannot become a successful record silently.

**Rationale:** Immutable versions preserve auditability and avoid last-writer data races.

### Decision: use a closed query IR

**Choice:** Nickel authors bounded selectors that lower to a closed Rust query IR. Supported initial predicates cover exact record or version identity, record class, operation type and method, instance, output name, schema identity, tag equality, producer run, workflow revision, outcome class, and bounded ordering or limit.

The query core receives an in-memory bounded candidate set and returns exact selected record references or deterministic diagnostics. It does not parse SQL, CEL, regex, scripts, closures, network sources, filesystem paths, or arbitrary payload code.

Structured field selection is allowed only through the declared resource schema and named bounded paths. File payloads remain opaque.

**Rationale:** Lattice needs composability without adding an ambient programming language to runtime data access.

### Decision: resolve dynamic selections before dispatch

**Choice:** `latest` and selector queries run during invocation preparation. A successful selection resolves to exact immutable version identities that become part of the downstream invocation plan and cache identity.

If the selection is empty, ambiguous, stale, sensitive, unauthorized, oversized, or changes during final pre-dispatch recheck, dispatch fails. The system does not silently choose another version.

**Rationale:** Reproducible execution requires exact inputs even when the author asks for a dynamic selection.

### Decision: separate garbage collection from orphan pruning

**Choice:** Lifetime classes include ephemeral, run, duration, and retained. Garbage collection applies declared expiry and version-count policy through a pure deletion plan and capability-confined shell.

Orphan pruning detects records whose operation type or instance is currently absent. It remains a separate preview-first command because branch switches, migrations, or temporarily unavailable packs can make valid data appear orphaned.

Retention and deletion receipts bind selected identities and outcomes. They do not claim secure erasure, remote deletion, or recovery impossibility.

**Rationale:** Explicit policy expiry is safer to automate than ownership inference.

### Decision: exclude secret material instead of storing and hiding it

**Choice:** Operation output contracts identify fields and outputs that cannot enter operational data. Credential values, authorization headers, environment values, secret-derived payloads, private host paths, and other prohibited classes fail record admission before payload publication.

Opaque file records require an explicit admitted sensitivity and destination policy. A redaction marker does not make known secret bytes safe to persist.

Query results and record diagnostics contain safe identities and bounded metadata only. They do not echo rejected payload fragments.

**Rationale:** Display redaction does not remove secrets from storage, indexes, backups, or later composition.

### Decision: keep Valence projection optional and one-way

**Choice:** Lattice can project selected record identity, producer lineage, schema role, content identity, sensitivity class, and non-claims into a saved Valence `RecordedOnly` artifact through an adapter.

The adapter never embeds operational payload by default. Valence validation does not make a record executable, current, true, authorized, retained, or release-eligible. Lattice never queries Valence as its operational source of truth.

**Rationale:** Evidence linkage and operational data ownership are separate stack roles.

## Storage Shape

The first implementation can extend the current redb store with separate metadata, lineage, tag, and version-order tables. Payload bytes remain behind a backend port and capability root.

Indexes are derived from canonical record metadata. The write shell must preserve these invariants:

- no visible admitted metadata points to an unverified payload;
- duplicate equivalent publication is idempotent;
- same identity with different bytes fails closed;
- index rows cannot widen tags, schema, sensitivity, or provenance;
- interrupted publication remains recoverable or explicitly uncertain;
- readers validate record and payload identity before returning data.

## Validation Strategy

Positive cases cover resource and file publication, repeated identical content from distinct invocations, exact version reads, latest resolution, tag and provenance queries, field selection, retention, idempotent garbage collection, and `RecordedOnly` projection.

Negative cases cover malformed schemas, wrong content hashes, duplicate identity conflicts, missing payloads, unsafe locators, oversized data, index drift, ambiguous latest ordering, stale pre-dispatch selection, secret material, retention conflicts, orphan false positives, and Valence overclaims.

Concurrency fixtures cover two writers producing versions for the same logical output. Crash fixtures cover death before payload write, after payload write, and before index commitment.

## Adversarial Review

The final audit must try to falsify these properties:

- `latest` cannot change after it is bound into a dispatch plan;
- equal content from different producers keeps distinct lineage;
- a sensitive value cannot enter a tag, summary, diagnostic, or index;
- garbage collection cannot act as implicit orphan pruning;
- query selectors cannot escape into arbitrary code or unbounded scans;
- Valence projection cannot become operational truth or a stronger verification role.

## Risks and Trade-offs

- Immutable versions increase storage. Explicit lifetime and version policies bound that growth.
- A small query IR offers less flexibility than CEL or SQL. It is easier to validate and evolve safely.
- Secret exclusion can reject useful provider outputs that mix credentials and ordinary data. Operation adapters must split safe facts from secret delivery.
- Crash-safe metadata and payload publication add shell complexity. The pure state plan remains separate and heavily tested.

## Non-Claims

This design does not prove payload truth, external-system state, semantic schema correctness, secure erasure, arbitrary crash durability, distributed consistency, remote completeness, access-control completeness, confidentiality of approved sensitive data, or release eligibility.
