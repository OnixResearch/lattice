## Context

A Lattice expression names a producer node and output port. The source text is an explicit data reference, but the current runner graph contains only normal graph edges.

The executor builds one expression context from every prior output. This creates an incidental order dependency that Choregraph cannot see.

Concurrent dispatch is unsafe until the admitted graph includes the same dependency meaning as expression evaluation.

## Success Contract

Completion requires exhaustive reference extraction, exact producer and port validation, deterministic dependency projection, confined runtime context, compatibility evidence, negative cases, and current identities.

These results are false completion:

- parsing expressions without adding dependency facts;
- retaining access to all prior outputs;
- accepting a reference because serial order currently makes it available;
- adding another graph admission or readiness algorithm in Lattice;
- mutating workflow source to hide a derived edge;
- treating a data dependency as proof that effects commute.

## Pure Expression Dependency Core

Add a Lattice-owned value with this semantic shape:

```text
ExpressionDependency {
  consumer_node,
  config_field_path,
  producer_node,
  producer_output_port,
}
```

The pure extractor receives one admitted workflow value. It returns ordered dependency facts or stable diagnostics.

The extractor performs no file, clock, process, network, persistence, credential, or runtime access.

## Exhaustive Field Inventory

Maintain one checked inventory of every field that can contain a Lattice expression. The inventory covers current shell, HTTP, file, image, agent, and future typed operation fields where expression syntax is allowed.

A new expression-bearing field without extraction coverage fails the completeness guard. A field that forbids expressions must reject expression syntax.

## Admission Rules

Each extracted reference must name:

- an existing producer node;
- an existing producer output port;
- a distinct consumer node;
- a supported expression shape;
- a dependency relation that remains acyclic after union.

Duplicate references normalize to one execution edge while retaining field-specific diagnostics. A direct graph edge and expression edge between the same nodes also normalize to one execution edge.

## Choregraph Projection

The pure Lattice adapter builds the canonical execution dependency set:

```text
execution_dependencies
  = data_flow_dependencies
  union expression_dependencies
```

Choregraph admits the resulting graph and remains the owner of cycle, order, readiness, lifecycle, and replay decisions.

The projection does not mutate the portable workflow source. The resulting machine and runner-plan identities bind the derived dependency set. A stale plan with another set fails closed.

## Runtime Context

Before node execution, Lattice builds an expression context from the exact admitted references for that consumer. It can also include declared workflow-bound inputs through their existing typed path.

The context does not include unrelated completed outputs. A missing required value is a deterministic pre-effect error.

## Compatibility

Valid existing workflows can retain their source bytes. Their derived machine identity can change because the admitted dependency graph becomes complete.

Record old and new projection identities for fixtures with expressions. Require an explicit compatibility disposition before release.

A workflow that relied on accidental serial order becomes invalid or gains an explicit derived dependency. It must not continue through an ambient output context.

## Testing

Positive fixtures cover direct references, multiple fields, duplicate references, existing graph edges, transitive order, Unicode values, cache hits, and portable round trips.

Negative fixtures cover missing nodes, missing ports, self-reference, downstream cycles, malformed syntax, unsupported fields, stale plans, context leakage, missing runtime values, and source mutation.

## Claim Boundary

This change establishes exact supported expression dependency projection. It does not prove arbitrary expression semantics, effect commutativity, provider behavior, workflow correctness, or runtime progress.
