## Why

Lattice expressions can read outputs from named nodes and ports. Current workflow validation checks expression syntax, but it does not add those references to the Choregraph dependency graph.

The executor supplies every prior output to expression evaluation. Serial topological execution can therefore hide an undeclared dependency. A dependency-driven runtime would expose missing values or order-dependent behavior.

Lattice must make every supported expression dependency explicit before it can dispatch independent nodes concurrently.

## What Changes

- Inventory every expression-bearing workflow field and reject unclassified additions.
- Extract typed producer, output-port, consumer, and field-path references in the pure workflow core.
- Reject unknown nodes, unknown ports, self-references, unsupported references, and dependency cycles before runtime selection.
- Merge data-flow edges and expression-reference edges into one canonical execution dependency graph before Choregraph admission.
- Restrict each expression context to exact admitted references and declared workflow inputs.
- Bind the derived dependency projection to the admitted machine and runner-plan identities.
- Add positive compatibility fixtures and negative hidden-dependency fixtures.

## Success Criteria

- Every runtime expression reference has one admitted dependency fact.
- Choregraph receives the complete data dependency graph that Lattice derives.
- Expression evaluation cannot read an unrelated prior node output.
- Invalid or cyclic references fail before credentials, attempts, or effects.
- Existing valid workflow source remains readable without source mutation.

## Dependencies

- Complete or supersede `isolate-workflow-domain-boundaries` before implementation changes shared workflow and expression types.
- Keep Choregraph as the graph admission and readiness owner.

## Non-Goals

- Inferring whether external effects commute.
- Scheduling nodes, assigning runtime capacity, or executing effects.
- Adding cyclic workflows or cross-round pipelines.
- Proving expression meaning beyond the supported parser and exact reference mapping.

## Impact

- **Core**: expression-reference extraction, dependency projection, and stable diagnostics.
- **Adapters**: Choregraph lowering, portable workflow conversion, and runner-plan identity.
- **Runtime**: confined expression contexts with no unrelated outputs.
- **Testing**: positive direct and transitive references plus negative malformed, stale, cyclic, and hidden dependencies.
