# Workflow domain boundaries

This document records ownership for workflow values and decisions. It is also the maintained migration inventory for `rat_nodegraph` use.

## Ownership

r[impl lattice.workflow_validation.domain_boundary.ownership]
r[impl lattice.workflow_validation.domain_boundary.ownership.complete]
r[impl lattice.workflow_validation.domain_boundary.docs]
r[impl lattice.workflow_validation.domain_boundary.docs.owners]

| Owner | Owned values and decisions | Allowed dependencies |
|---|---|---|
| Lattice workflow core | Workflow metadata, node and port identities, typed port declarations, product configuration, aggregate mutation, and domain errors | Can call pure Choregraph conversion and admission code. It must not use Subwayrat types or read a clock. |
| Choregraph | Graph admission, lifecycle, replay, order, and machine decisions | Receives explicit Lattice values through `src/workflow_format/choregraph.rs`. It does not own product configuration or effects. |
| Subwayrat | Canvas gestures, selection, layout interaction, and render state | Stays under `src/tui/`. `src/tui/graph.rs` converts to and from Lattice-owned graph values. |
| Persistence adapters | Supported Nickel, JSON, and redb record shapes | Decode into Lattice values and encode from Lattice values. A malformed record returns an adapter error before state changes. |
| Imperative shells | Clock reads, file and database I/O, process work, providers, and terminal interaction | Read one timestamp for each requested metadata mutation and pass it into the core as `WorkflowTimestamp`. |

Dependencies point from shells and adapters toward the workflow core. UI, storage, and provider types do not enter core fields, arguments, results, or errors.

## Subwayrat migration inventory

r[impl lattice.workflow_validation.domain_boundary.inventory]
r[impl lattice.workflow_validation.domain_boundary.inventory.complete]

Before this migration, selected non-TUI paths used `rat_nodegraph::model` identities, ports, edges, graphs, and graph errors. The table records their replacement and current boundary.

| Selected path | Prior Subwayrat values | Current owner and value |
|---|---|---|
| Workflow aggregate and mutations | `Graph`, `GraphError`, `NodeId`, `PortId`, `PortSpec` | Lattice: `WorkflowGraph`, `GraphError`, `NodeId`, `PortId`, `PortSpec` |
| Validation and Choregraph lowering | Graph nodes, ports, edges, and identities | Lattice values; the pure Choregraph lowerer maps admitted values |
| Execution and child-workflow planning | `NodeId`, `PortId` | Lattice `NodeId` and `PortId` |
| Expression references | `PortId` | `lattice_core::expressions::ExpressionPortId` at the reusable core boundary, then Lattice `PortId` |
| Replay and run history | `NodeId`, `PortId` | Lattice `NodeId` and `PortId` |
| Nickel and JSON workflow records | Serialized Subwayrat `Graph` | Version-compatible records decoded into `WorkflowGraph` |
| redb workflow persistence | Serialized Subwayrat `Graph` | Version-compatible records decoded into `WorkflowGraph` |
| TUI canvas | Subwayrat graph and view values | Subwayrat-owned values under `src/tui/`; explicit adapter conversion at workflow mutation and rendering boundaries |

The maintained source guard scans non-TUI Rust sources for `rat_nodegraph` leakage. It also scans declared workflow-core files for ambient clock reads:

```sh
scripts/check-workflow-domain-boundaries.rs
scripts/check-workflow-domain-boundaries.rs --self-test
```

`scripts/check.rs` runs both checks. Octet also keeps `ambient_clock` active. The former workflow-core clock findings are removed by explicit timestamp inputs rather than by a baseline exception.

## Time contract

`Workflow::new`, `WorkflowMeta::new`, and `Workflow::touch` require `WorkflowTimestamp`. The type wraps a valid `chrono::DateTime<Utc>`.

CLI, import, and TUI shells can read current time. Tests and replay paths supply fixed or recorded values. Equal explicit inputs produce equal metadata.

## Claim boundary

r[impl lattice.workflow_validation.domain_boundary.final_checks]
r[impl lattice.workflow_validation.domain_boundary.final_checks.claims]

These boundaries establish selected dependency, time-input, and compatibility facts. They do not prove whole-workflow correctness, provider correctness, external-effect safety, or release readiness.
