# Design: Isolate workflow domain boundaries

## Context

The workflow aggregate mixes three separate models:

1. Lattice product configuration and aggregate mutation.
2. Choregraph graph and machine admission.
3. Subwayrat graph editing and rendering.

`WorkflowMeta::new` and `Workflow::touch` also read the host clock. As a result, direct core calls do not have complete explicit inputs.

## Success Contract

Completion requires a deterministic workflow aggregate over explicit values. Lattice-owned signatures contain no Subwayrat types and read no clock.

The adapters must preserve supported workflow files, records, replay behavior, and TUI editing. Choregraph must remain the graph-decision owner.

Renaming imports, hiding aliases, or adding a clock trait inside the core is false completion.

## Decisions

### Decision: Lattice owns the product workflow model

**Choice:** Add Lattice-owned identifiers, port declarations, graph-edit requests, mutation plans, and errors for product workflows.

This model binds node configuration, credentials, runners, triggers, and product metadata. It does not independently define Choregraph lifecycle or machine semantics.

**Rationale:** Product rules need stable application types. UI component types must not define product contracts.

### Decision: Choregraph remains the graph authority

**Choice:** Convert admitted Lattice workflow values into the pinned Choregraph contract through one pure adapter.

The adapter maps explicit values and returns Lattice-owned admission diagnostics. It must not expose Choregraph transport or frontend errors across the application boundary.

**Rationale:** Lattice owns product meaning. Choregraph owns graph and machine decisions.

### Decision: Subwayrat is an inbound and outbound UI adapter

**Choice:** Keep `rat_nodegraph` model and view types inside TUI adapter modules.

The adapter converts UI gestures and snapshots into Lattice mutation requests. It converts accepted Lattice state into render values.

**Rationale:** Subwayrat provides UI behavior. It does not own workflow state or mutation policy.

### Decision: Time is an explicit shell fact

**Choice:** Workflow creation and metadata updates receive a validated UTC timestamp.

CLI, TUI, import, and other shells obtain current time once. Replay and tests supply recorded or fixed values.

**Rationale:** Time changes domain output and must be an explicit input.

### Decision: Compatibility uses explicit boundary records

**Choice:** Keep supported Nickel, JSON, and redb shapes behind conversion functions.

Loaders convert boundary records into admitted Lattice values. Writers project admitted values back to the supported versioned shape.

**Rationale:** Internal ownership can improve without silently changing durable formats.

### Decision: Migrate in bounded slices

**Choice:** Introduce application types and adapters before call-site migration.

Migrate workflow construction, validation, expression evaluation, execution, replay, history, persistence, and TUI use in reviewed groups.

**Rationale:** The workflow model has many consumers. Bounded slices keep compatibility failures local.

## Required Flow

```text
Nickel, JSON, redb, CLI, or TUI input
  -> boundary adapter
  -> Lattice-owned workflow command
  -> deterministic workflow core
  -> accepted state, error, or mutation plan
  -> Choregraph admission when graph meaning is required
  -> imperative shell
  -> persistence, runtime, or UI adapter
```

## Error Ownership

The workflow core owns invalid aggregate state, unknown nodes, incompatible ports, and rejected mutations.

Adapters own malformed boundary data, unsupported schema versions, UI conversion failures, storage failures, and Choregraph cohort incompatibility.

The shell translates these classes without turning an adapter failure into a domain rejection.

## Test Design

Positive tests cover explicit timestamp creation, metadata updates, graph edits, Choregraph admission, Subwayrat round trips, persistence, and replay.

Negative tests cover missing or invalid timestamps, stale edits, unknown nodes, incompatible ports, adapter drift, malformed records, and legacy record failures.

Compile-time or source checks reject `rat_nodegraph` types and ambient clock reads in declared workflow-core scopes.

## Risks and Trade-offs

- The migration touches many call sites. Compatibility fixtures must pass after each slice.
- Application-owned graph values add conversion code. They prevent UI types from controlling product contracts.
- Choregraph conversion can expose cohort drift. The adapter must fail with a distinct infrastructure error.
- Existing active TUI work also edits workflow metadata. This change must define stable prerequisites and avoid duplicate field policy.

## Claim Boundary

This change establishes explicit dependency and time-input boundaries for selected workflow paths. It does not prove graph correctness or safe external effects.
