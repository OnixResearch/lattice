# Design: Control-flow and subworkflow nodes

## Architecture

Add control-flow node kinds as data in `NodeConfig` and behavior behind node-kind providers and runtimes once those boundaries exist.

```text
Workflow graph
  ├─ Condition node ─ pure predicate admission over typed inputs
  ├─ Merge node     ─ pure fan-in policy admission
  └─ Subworkflow    ─ runner-shell invocation of saved workflow
```

The functional core owns predicate parsing, branch selection, merge readiness, subworkflow interface checks, and recursion admission. The imperative shell loads referenced workflows, invokes nested runner execution, persists child-run records, and emits parent/child event links.

## Condition nodes

A condition node has one value input and one or more named output branches. Initial predicates should be simple, typed, and deterministic: presence, equality, string contains, numeric comparison where type tags support it, JSON pointer existence/equality, and boolean truthiness. Predicate evaluation returns a selected branch list; unselected branch outputs are absent, not empty strings.

## Merge nodes

A merge node declares a fan-in policy:

- `all`: wait for all required inputs;
- `first`: complete when the first input arrives;
- `last`: select the latest input by deterministic runner sequence;
- `collect`: emit an ordered JSON array of available input summaries.

Merge readiness is pure and depends only on upstream terminal states and available typed values.

## Subworkflow nodes

A subworkflow node references a saved workflow by name or stable workflow id plus an interface mapping. The parent runner shell loads the child workflow, validates it, binds parent input values to child entry inputs, executes the child through the same runner shell, and maps declared child outputs back to parent output ports.

Recursive references are rejected by a pure ancestry check before execution. Parent and child run events share correlation ids so TUI, headless, and history can display nested execution without flattening away context.

## Format and compatibility

Nickel import/export gains optional control-flow config fields. Existing files that lack these fields import unchanged. Exported control-flow nodes include predicate definitions, branch labels, merge policies, subworkflow references, and interface mappings.

## Testing

Use pure tests for predicate evaluation, merge readiness, and recursion admission. Use fake repositories/runners for subworkflow positive and negative tests. Integration tests prove TUI/headless compatibility and Nickel round trips.
