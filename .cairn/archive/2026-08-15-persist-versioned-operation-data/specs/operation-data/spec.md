# Operation Data

## ADDED Requirements

### Requirement: Operation outputs use typed immutable records

r[lattice.operation_data.records] Lattice MUST represent admitted cross-run operation outputs as versioned structured resources or opaque files with exact producer, schema, content, storage, sensitivity, lifetime, and non-claim facts.

#### Scenario: Structured resource is admitted

r[lattice.operation_data.records.resource]
- GIVEN an operation output matches its declared resource schema, canonical form, byte limit, sensitivity policy, producer identities, lifetime, and required non-claims
- WHEN record admission runs
- THEN Lattice MUST emit one deterministic resource record and domain-separated BLAKE3 identity
- AND the resource payload MAY be selected only through its declared structured schema.

#### Scenario: Opaque file is admitted

r[lattice.operation_data.records.file]
- GIVEN an operation output declares an opaque file role, media class, byte length, content BLAKE3, safe backend locator, sensitivity policy, producer identities, lifetime, and required non-claims
- WHEN record admission runs
- THEN Lattice MUST emit one deterministic file record
- AND query evaluation MUST treat the file payload as opaque.

#### Scenario: Record facts are malformed

r[lattice.operation_data.records.invalid]
- GIVEN a record has an unsupported version, unknown field, missing producer, wrong schema, wrong hash, unsafe locator, oversized payload, contradictory sensitivity, or missing non-claim
- WHEN admission runs
- THEN Lattice MUST reject the record before publication or indexing
- AND it MUST NOT reinterpret opaque bytes as a structured resource.

### Requirement: Every admitted output appends an immutable version

r[lattice.operation_data.versions] Lattice MUST append one immutable version for each admitted output and MUST preserve exact retrieval and producer lineage without updating prior versions in place.

#### Scenario: New output creates another version

r[lattice.operation_data.versions.append]
- GIVEN one logical operation output already has an admitted version and a later invocation produces another valid output
- WHEN publication completes
- THEN Lattice MUST retain both exact versions and their distinct invocation lineage
- AND identical content bytes from distinct invocations MUST NOT erase producer attribution.

#### Scenario: Duplicate or conflicting publication occurs

r[lattice.operation_data.versions.conflict]
- GIVEN publication repeats equivalent record and payload facts or presents different bytes under an existing version identity
- WHEN the publication shell rechecks state
- THEN the equivalent publication MUST be idempotent and the conflicting publication MUST fail closed
- AND existing payload, metadata, lineage, and indexes MUST remain unchanged after conflict.

#### Scenario: Publication outcome is uncertain

r[lattice.operation_data.versions.uncertain]
- GIVEN the process stops between payload verification, metadata append, and index commitment
- WHEN recovery opens the store
- THEN Lattice MUST repair from exact verified facts or report an explicit incomplete or uncertain publication
- AND it MUST NOT invent an admitted version or silently discard conflicting evidence.

### Requirement: Operation data queries use a closed bounded IR

r[lattice.operation_data.query] Lattice MUST lower typed selectors into a closed Rust query IR and MUST evaluate them deterministically over bounded metadata without executing user code or ambient effects.

#### Scenario: Selector resolves exact resource versions

r[lattice.operation_data.query.valid]
- GIVEN a selector uses supported exact identity, class, operation, method, instance, output, schema, tag, producer, workflow revision, outcome, ordering, and limit predicates
- WHEN bounded query reduction runs
- THEN Lattice MUST return deterministic exact version references in stable order
- AND structured field selection MUST remain within declared resource-schema paths.

#### Scenario: Dynamic selection becomes an exact invocation input

r[lattice.operation_data.query.bound]
- GIVEN an operation input requests the latest or a bounded selected prior resource
- WHEN invocation preparation resolves the selector
- THEN the canonical invocation plan MUST bind the exact selected version identity and content identity
- AND a changed, empty, ambiguous, stale, sensitive, or unauthorized selection MUST block dispatch rather than choose another version.

#### Scenario: Unsupported query behavior is requested

r[lattice.operation_data.query.denied]
- GIVEN a selector requests SQL, CEL, scripts, closures, regex, arbitrary payload code, file payload fields, unbounded scans, network data, or filesystem data
- WHEN query validation runs
- THEN Lattice MUST reject the selector with deterministic diagnostics
- AND it MUST execute no query-supplied code or ambient effect.

### Requirement: Retention and orphan pruning remain separate

r[lattice.operation_data.retention] Lattice MUST enforce declared ephemeral, run, duration, retained, and version-count policies through explicit retention plans and MUST keep inferred orphan pruning preview-first and separate.

#### Scenario: Declared lifetime expires

r[lattice.operation_data.retention.gc]
- GIVEN an admitted record exceeds its exact lifetime or version-count policy
- WHEN garbage-collection planning and explicit policy execution run
- THEN Lattice MUST select only records covered by that policy and emit an identity-bound outcome
- AND equivalent repeated execution MUST remain idempotent.

#### Scenario: Owner is temporarily absent

r[lattice.operation_data.retention.orphan_preview]
- GIVEN a branch switch, migration, unavailable operation pack, or catalog edit makes a record owner appear absent
- WHEN orphan pruning runs without explicit execution
- THEN Lattice MUST report a preview and mutate no payload, metadata, lineage, or index
- AND ordinary garbage collection MUST NOT delete the record solely because its owner is absent.

### Requirement: Plaintext secret material is excluded from operation data

r[lattice.operation_data.confidentiality] Lattice MUST reject plaintext credentials, authorization data, environment values, secret-derived payloads, private host paths, and undeclared sensitive data before operation-data publication, indexing, querying, or receipt projection.

#### Scenario: Safe non-secret output is published

r[lattice.operation_data.confidentiality.safe]
- GIVEN an operation output passes its declared non-secret or explicitly admitted sensitive-data policy and contains no prohibited secret material
- WHEN record admission and publication run
- THEN Lattice MAY persist the bounded payload and safe metadata
- AND display and query paths MUST preserve the declared sensitivity class.

#### Scenario: Secret material reaches a record draft

r[lattice.operation_data.confidentiality.rejected]
- GIVEN a payload, tag, summary, schema field, locator, diagnostic, or index value contains credential material, an authorization header, an environment value, a secret-derived fragment, or a prohibited private path
- WHEN record admission runs
- THEN Lattice MUST reject publication without echoing the prohibited value
- AND redaction during later display MUST NOT substitute for rejection from storage.

### Requirement: Valence projection is optional and recorded-only

r[lattice.operation_data.valence_projection] Lattice MAY project selected safe operation-record metadata into a Valence-compatible `RecordedOnly` artifact while retaining operational storage, selection, freshness, authorization, and execution authority.

#### Scenario: Selected metadata projects safely

r[lattice.operation_data.valence_projection.valid]
- GIVEN an admitted record is selected for evidence linkage and its identity, lineage, schema role, sensitivity class, content identity, and non-claims are safe to disclose
- WHEN projection runs
- THEN Lattice MUST emit a deterministic saved artifact draft without operational payload by default
- AND Valence validation MUST NOT make the record executable, true, current, authorized, retained, or release-eligible.

#### Scenario: Projection requests a stronger role or authority

r[lattice.operation_data.valence_projection.denied]
- GIVEN a projection embeds prohibited payload, requests a property role without separate evidence, omits non-claims, cites stale identity, or asks Valence to serve runtime data
- WHEN projection validation runs
- THEN Lattice MUST reject the projection
- AND the rejected artifact MUST NOT satisfy an operation input, retention decision, authority gate, lifecycle gate, or release claim.
