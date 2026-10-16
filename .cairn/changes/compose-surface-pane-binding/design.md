## Context

Lattice runs typed graph workflows and owns workflow meaning inside this repo. Tile owns agent-pane observation surfaces and readiness receipts. The two products have no typed connector today. Operators build ad hoc message shapes that leave no provenance.

The Squeak Etoys interaction is the reference for the target shape. This change adapts only the composition discipline, not Squeak's ambient mutation or unversioned change.

## Decisions

### Decision: Own the connector in Lattice

**Choice:** Add a narrow no-std binding family under Lattice and an injected Tile observation adapter.

**Rationale:** Lattice owns workflow and trigger composition. Tile keeps observation truth. A neutral connector crate would add a sibling repo without an owner. Lattice already hosts input-projection adapters.

### Decision: Receipts are metadata only

**Choice:** Each activation emits a metadata-only BLAKE3 receipt under Valence evidence rules.

**Rationale:** Metadata-only receipts keep the claim boundary honest and keep the core store-free.

### Decision: No sibling-path dependency

**Choice:** Consume the Tile observation projection through an injected adapter bound to an exact reviewed revision.

**Rationale:** The stack rule forbids sibling-path dependencies as durable product behavior.

### Decision: No clocks or product state in the core

**Choice:** The core decides from supplied values only.

**Rationale:** Deterministic plans keep the pure core testable and replay-safe.

## Risks / Trade-offs

- Cross-repo drift: mitigated by the recorded revision contract.
- Scope creep into an ETL: bounded by the non-goals.
- Over-claim of workflow success: mitigated by the metadata-only receipt boundary.
