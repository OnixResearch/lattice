## Why

Lattice's TUI and typed Nickel workflow files share the persisted `Workflow` model, but they are not equivalent authoring surfaces. The TUI cannot create HTTP or file nodes and exposes only a subset of node configuration, workflow metadata, ports, runner policy, triggers, merge policy, and subworkflow mappings. Untouched hidden fields often survive store save/load, but preservation is not authoring parity.

Two mutation defects also violate the shared-model boundary. Canvas deletion removes only the graph node and can leave orphan configuration, credentials, and runner state that validation or Nickel import rejects. Condition branch editing zips new branches with existing output ports, so branch-count changes can leave missing ports or dangling edges.

## What Changes

- Define a pure authoring-capability catalog for every field in the current portable workflow schema: metadata, all eight built-in node kinds, typed ports, topology, per-node credentials, runner settings/policies, and manual/schedule/webhook triggers.
- Extend guided TUI creation to HTTP and file nodes and expose provider-backed, typed property editors for every current built-in node configuration field rather than scattered node-specific matches.
- Add workflow, port, runner-policy, credential, and trigger editing surfaces with explicit typed parsers, validation, cancellation, and recoverable errors; no raw unvalidated source replacement counts as parity.
- Centralize node deletion in a pure workflow lifecycle plan plus thin mutation shell that removes the graph node, incident edges, config, credentials, runner settings, selection/edit state, and any trigger-owned reference that cannot remain valid.
- Replace condition branch/port `zip` mutation with a pure deterministic reconciliation plan that preserves stable ports and edges for unchanged branch labels, creates ports for new labels, removes ports and incident edges for deleted labels, and rejects duplicate/invalid edits atomically.
- Add an exhaustive parity matrix and round-trip fixtures proving every authorable field reaches equivalent `Workflow` and Nickel representations and every invalid edit preserves prior state.

## Success Criteria

- Operators can create every current portable built-in node kind and edit every portable workflow field through typed TUI flows.
- A TUI-authored workflow containing metadata, all node kinds, custom typed ports/edges, credentials, runner policies, control-flow mappings, and every trigger kind exports to Nickel and imports with equivalent meaning.
- Deleting any node leaves no orphan config, credential, runner, port, edge, edit, or invalid trigger state; save/export/import validation remains clean.
- Increasing, decreasing, renaming, reordering, duplicating, or cancelling condition branches has deterministic, atomic topology behavior with stable unchanged port identities.
- Positive and negative tests cover every capability row, parser boundary, cancellation path, deletion state class, condition topology transition, persistence round trip, and constrained-terminal rendering path.

## Out of Scope

- Editing arbitrary future plugin schemas that are not present in the current portable workflow contract.
- Replacing typed forms with an unrestricted embedded text editor, evaluating arbitrary Nickel inside the TUI, or bypassing workflow validation.
- Changing runtime execution semantics, adding new node kinds, resolving credentials, or implementing the separate SecretSpec/SOPS provider follow-up.
- Claiming that UI layout or interaction is byte-identical across terminals or that parity proves workflow correctness, external effect safety, or production readiness.

## Impact

- **TUI:** add-node flow, property/workflow/trigger editors, help/palette metadata, state transitions, rendering, and tests.
- **Core:** pure authoring catalog/parsers, deletion planning, condition topology reconciliation, and workflow lifecycle APIs.
- **Format:** parity matrix and cross-surface round-trip tests; the portable Nickel schema remains the semantic reference.
- **Compatibility:** existing workflows and hidden imported fields remain preserved; current shortcuts continue to work while new fields are discoverable through typed catalog-driven actions.
