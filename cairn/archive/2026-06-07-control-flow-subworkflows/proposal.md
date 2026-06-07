# Proposal: Control-flow and subworkflow nodes

## Summary

Add first-class conditional routing, merge, and subworkflow invocation nodes so workflows can express branching and composition without shell-only workarounds. The feature builds on the generic runner plan by keeping branch admission pure and routing subworkflow execution through the same runtime/event surfaces as normal runs.

## Problem

Current workflows are linear DAGs with typed edges, but authors cannot express common automation control flow directly. Conditional behavior must be embedded in shell commands, merge semantics are implicit, and reusable workflow composition requires copying nodes instead of invoking a saved workflow with typed inputs and outputs.

## Goals

- Represent conditional branch nodes with typed predicates over upstream values.
- Represent merge nodes with deterministic fan-in policies.
- Invoke saved workflows as subworkflow nodes through the runner shell rather than bespoke executor branches.
- Preserve typed ports, validation, cache identity, credentials, runner events, and run history across nested execution.
- Import and export control-flow metadata through typed Nickel workflow files.

## Non-Goals

- Add arbitrary scripting languages for predicates.
- Introduce cycles or imperative loops in the workflow graph.
- Replace the generic runner or runtime registry work.
- Make subworkflows share mutable state with parent workflows.

## Success Criteria

- Valid conditional and merge workflows run deterministically through TUI, headless, and scheduler entrypoints.
- Invalid predicates, missing branches, incompatible merge inputs, and recursive subworkflow references fail validation before side effects.
- Subworkflow outputs are routable through declared output ports and appear in ordered runner events and run history.
- Existing workflows remain compatible without adding control-flow metadata.
