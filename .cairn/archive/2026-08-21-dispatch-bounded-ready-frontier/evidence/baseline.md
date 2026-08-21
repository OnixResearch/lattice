# Serial executor baseline

<!-- r[verify lattice.workflow_execution.ready_frontier.baseline] -->

Recorded before the ready-frontier executor change on 2026-08-20.

## Behavior

- The executor iterated one admitted topological list.
- One node completed before the next node started.
- Start order, completion order, and returned result order were the same topological order.
- `AppConfig.max_parallelism` defaulted to four but did not affect executor dispatch.
- Cache hits completed in place during the serial walk.
- Expression dependencies were projected before graph admission.
- Product effects required the current Choregraph authority session.
- Operation attempts reserved durable identity before backend dispatch.
- Post-effect persistence failure remained an unknown attempt outcome.
- Recovery did not infer retry safety from process failure alone.

## Baseline rails

The pre-change focused baseline passed these suites:

- `test_runner_core`;
- `test_expression_dependencies`;
- `test_choregraph_effect_authority`;
- `test_choregraph_adoption`;
- `test_control_flow`;
- `test_executor`;
- `test_subworkflow_execution`;
- Choregraph migration tests.

These tests covered graph order, cache identity, hidden expression dependencies, current authority, backend attempts, unknown outcomes, and migration compatibility.

## Critical-path limitation

The serial executor made total elapsed time approach the sum of independent node durations. It did not release a fast branch child while an unrelated slow sibling was running.

The baseline did not record a portable latency target. Host load, process startup, and adapter behavior are outside the deterministic graph contract.
