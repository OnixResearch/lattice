## Context

Lattice currently stores executable product configuration in a closed `NodeConfig` enum. `NodeKindProvider` centralizes metadata and validation for built-in kinds. `NodeRunnerSettings` can select generic runtime bindings, but it does not define a reusable external operation contract or data-only instance model.

The desired boundary has three distinct identities:

1. An operation type defines reusable methods, schemas, capability classes, runtime requirements, and non-claims.
2. An operation instance binds one exact type revision to target facts, tags, and credential references.
3. An invocation binds one instance, method, arguments, runtime policy, and output contract for one run.

Swamp is a product reference for this separation. The implementation remains clean-room and uses Lattice-owned Rust and Nickel contracts.

## Success Contract

Completion requires one production path from validated Nickel catalog data to strict Rust DTOs, pure invocation admission, existing runtime dispatch, typed result mapping, and preview-first Nickel promotion.

The same invocation plan must drive direct execution and promoted workflow execution. Separate implementations, hidden shell expansion, schema-only validation, or post-effect validation are false completion.

## Decisions

### Decision: Nickel owns authored operation contracts

**Choice:** Add typed Nickel contracts for operation types and instances. Exported or directly decoded values are revalidated by Rust with unknown-field rejection, named limits, stable diagnostics, and domain-separated BLAKE3 identities.

An operation type includes:

- canonical type ID and schema version;
- human description and safe tags;
- exact type revision identity;
- one or more named methods;
- method argument and output contracts;
- preflight declarations;
- capability and side-effect classes;
- runtime-kind and runtime-key requirements;
- credential binding names and field policy;
- timeout, input, output, and artifact bounds;
- required non-claims.

**Rationale:** Agents can author and review structured Nickel without receiving executor internals or ambient authority.

### Decision: Instances remain data-only

**Choice:** An operation instance can bind target facts, safe labels, method defaults, tags, and opaque credential references. It cannot define methods, change schemas, widen capabilities, select undeclared runtime kinds, inject commands, or replace non-claims.

The instance identity binds the exact type revision and normalized instance data. Display aliases remain outside authority identity.

**Rationale:** Reusable behavior and environment-specific data change for different reasons and need separate review.

### Decision: Admission is a pure functional core

**Choice:** Pure Rust code validates type, instance, method, arguments, output contract, capability policy, runtime binding facts, credential-reference shape, and declared bounds. It returns either one deterministic invocation plan or ordered diagnostics.

The core performs no Nickel evaluation, file read, process execution, network access, credential resolution, clock read, persistence, logging, or rendering.

**Rationale:** Operation policy must be testable without standing up the external system.

### Decision: direct and workflow execution share one plan

**Choice:** `lattice operation run` loads one admitted type and instance, binds arguments, builds the canonical invocation plan, reserves the run identity, then dispatches through the existing runtime registry and credential boundary.

Workflow import lowers an operation node to the same plan shape and typed ports. Choregraph handles graph and lifecycle facts. Lattice retains runtime, effects, credentials, budgets, persistence, diagnostics, and evidence.

A direct run does not create a committed operation instance or workflow automatically.

**Rationale:** One execution plan prevents direct-run behavior from drifting away from workflow behavior.

### Decision: promotion is preview-first and identity-preserving

**Choice:** `lattice operation promote` consumes an exact admitted invocation or persisted invocation record. Pure planning emits deterministic Nickel source or a blocker. The default command prints the destination, content BLAKE3, operation identities, and safe diff summary without writing.

Explicit execution uses capability-relative publication and refuses an existing destination, stale source invocation, unrepresentable value, plaintext credential material, missing type revision, or changed output contract.

Promotion writes opaque credential references only. It does not write resolved secret values, provider outputs, raw logs, or ambient environment values.

**Rationale:** The one-off path is useful only when an operator can convert it into reviewable, reproducible source without losing authority facts.

### Decision: use Cairn as the first read-only pilot

**Choice:** Add one reviewed adapter with read-only methods such as repository status and lifecycle validation. The adapter invokes an explicitly configured Cairn executable through Bounded Exec with a capability-confined repository root and named output/deadline bounds.

The pilot rejects mutation methods, unrestricted extra arguments, ambient current-directory discovery, unbounded output, malformed JSON, and any attempt to infer lifecycle completion from process success alone.

**Rationale:** Cairn supplies deterministic structured output and exercises real external-tool integration without production infrastructure mutation.

### Decision: expose metadata without granting execution

**Choice:** Operation discovery returns safe type, method, schema, capability, and non-claim metadata. An agent-facing adapter can map this metadata into Animus tool descriptors only when the host injects the matching Lattice executor.

A description, schema, skill, or model selection never grants execution authority.

**Rationale:** Discovery and execution need separate trust decisions.

## Validation Strategy

Positive cases cover valid type and instance catalogs, read-only Cairn methods, direct execution, exact result typing, and deterministic promotion.

Negative cases cover duplicate IDs, stale revisions, unknown fields, undeclared methods, schema mismatches, capability widening, missing runtimes, malformed credential references, timeouts, truncated output, wrong output types, existing destinations, stale promotion records, and secret leakage.

The production planner and runtime adapter must appear in the tests. Mock-only alternate planners do not satisfy the change.

## Adversarial Review

The final audit must reject these false paths:

- embedding arbitrary commands in instance data;
- loading executor code while listing metadata;
- resolving credentials during planning or display;
- trusting executor-reported output without contract validation;
- promoting a run after its type or output contract changes;
- converting process success into external correctness;
- adding dynamic TypeScript or YAML compatibility as the canonical path.

## Risks and Trade-offs

- A generic contract can become an unsafe plugin interface if runtime and capability bindings are weak. Exact bindings and fail-closed admission remain mandatory.
- Promotion can preserve stale values that were intentionally one-off. The preview identifies the exact source invocation and requires explicit execution.
- The closed `NodeConfig` model may need an additive generic operation variant. Compatibility and TUI parity remain explicit follow-up work.
- The Cairn pilot proves the operation boundary only. It does not prove broad infrastructure suitability.

## Non-Claims

This design does not prove external-system truth, operation implementation correctness, process sandboxing, hermeticity, credential freshness, remote effect uniqueness, workflow correctness, production readiness, or release eligibility.
