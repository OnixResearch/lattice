## Goals

- Measure whether bounded incremental recomputation helps representative Lattice workflows.
- Keep one independent full-recomputation oracle.
- Preserve exact workflow, authority, and operation identities.
- Produce an accept, reject, or insufficient-evidence decision.

## Pilot boundary

The pilot is test-only. Production execution continues to use existing Lattice behavior.

The pure pilot core receives a validated DAG, prior node observations, changed source facts, explicit limits, and a selected policy.

It returns an affected-node plan and bounded structural measurements. It performs no execution, storage, clock, queue, process, or network operation.

## Decisions

### Decision: Full recomputation remains the correctness comparator

**Choice:** Every pilot case compares incremental output with an independent complete recomputation over the same explicit facts.

**Rationale:** The candidate cannot validate itself.

### Decision: Invalidation follows declared dependencies only

**Choice:** The pilot starts from changed input or node identities and follows admitted dependency edges to a bounded frontier.

**Rationale:** Hidden dependencies make selective recomputation unsafe.

### Decision: Authority facts participate in identity

**Choice:** Capability context, operation contract, workflow revision, and policy identity participate in candidate keys and invalidation.

**Rationale:** Equal data does not make two authorized executions interchangeable.

### Decision: Measurements are structural and bounded

**Choice:** The pilot records visited nodes, planned nodes, reused exact results, divergent outputs, bounded durations, and resource observations.

Clock and resource observations come from the shell. They do not enter the pure decision.

**Rationale:** A pilot needs comparable facts without hiding nondeterminism in the core.

### Decision: Policy owns acceptance thresholds

**Choice:** A typed Nickel profile owns corpus requirements, bounds, minimum benefit, maximum overhead, and divergence policy.

**Rationale:** Numeric acceptance values must not become unexplained source literals.

### Decision: Production adoption requires another Cairn

**Choice:** This pilot can return `AdoptCandidate`, `RejectCandidate`, or `InsufficientEvidence`.

A later change must define production state, transactions, retries, observability, migration, and rollback.

**Rationale:** Pilot success does not authorize production effects.

## Corpus

The corpus includes deterministic chain, diamond, fan-out, fan-in, deep dependency, sparse-change, multi-source, and no-change cases.

Negative cases include cycles, missing nodes, stale graph identity, crossed authority, hidden-effect nodes, exceeded bounds, and candidate-oracle divergence.

## Evidence

Each run binds source revisions, corpus identity, workflow graph, changed facts, policy, candidate implementation, baseline implementation, and result identities.

The receipt records aggregate measurements and verdicts. It omits credentials, raw private payloads, and unbounded node output.

## Non-claims

Agreement proves parity only for the supplied corpus. It does not prove general workflow correctness or future performance.

A speed improvement does not prove safe result reuse. A cache hit does not prove current node truth.
