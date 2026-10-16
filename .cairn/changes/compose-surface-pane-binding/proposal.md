## Why

Lattice owns typed graph workflows and trigger composition. Tile owns daemon-managed panes and agent-pane observation surfaces. Operators cannot currently bind a live pane observation to a Lattice workflow through a typed, recorded contract. Each binding is ad hoc and leaves no provenance.

The Squeak Etoys model shows the target shape: a person grabs a live object and composes behavior with visible structure. This change adapts that interaction without Squeak's ambient mutation. A pane observation is one typed artifact. A workflow binding is a second typed artifact. The receipt that connects them is the evidence.

## What Changes

- Add a pure no-std binding family: composition DTOs, pane-observation admission, binding-plan decisions, canonical binding keys, and metadata-only receipt projections. r[lattice.compose_surface.identity]
- Admit pane-observation payloads against an explicit schema with missing and unknown fields rejected. r[lattice.compose_surface.admission]
- Produce deterministic binding plans from supplied values only, without clocks or environment reads. r[lattice.compose_surface.plan]
- Publish one metadata-only receipt per accepted binding activation. r[lattice.compose_surface.receipt]
- Record the cross-repo contract with the published Tile agent-pane observation surface. r[lattice.compose_surface.dependency]

## Impact

- **Files**: binding core crate, host shell wiring, Tile observation adapter, tests, docs, and this lifecycle package.
- **Testing**: workspace tests, Clippy, formatting, Cairn validation and gates, and Nix checks.

## Dependencies

Cross-repo contract: published Tile agent-pane observation projection, consumed through an injected adapter at an exact reviewed revision.

Evidence convention: Valence metadata-only receipt rules. Cairn lifecycle in this repo.

## Non-goals

- Do not change workflow execution semantics.
- Do not prove observation truth, workflow correctness, or effect success.
- Do not move Tile's observation authority into Lattice.
- Do not render or persist to product stores from the core.
- Do not read clocks in the core.
