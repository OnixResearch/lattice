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

### Decision: Consume Tile's existing metadata-only workspace inspection

**Choice:** Consume `tile-sdk::TileClient::inspect_workspace` and
`tile-protocol::WorkspaceInspectResponse` at exact producer revision
`5e703953840bff72a91fec3e928682e9efb607ab`. This producer is covered by Tile's
archived `add-native-windowed-live-control-plane` change. The adapter checks the
producer's complete bounded identity, expected-revision, and redaction contract
before narrowing one pane to lifecycle/focus metadata. It does not consume
`PaneReadResponse`, terminal content, detection strings, or perception output.

**Rationale:** The existing accepted producer contract provides the required observation
without giving Lattice screen-reading authority or copying producer implementation.
The daemon authority digest and socket connection are explicit injected host
inputs; the adapter's pinned contract revision is not attestation of the daemon
binary or a proof that the pane observation is true.
Exact anonymous source acquisition is a separate dependency-admission gate; local
source and archived producer requirements do not establish public availability.

### Decision: Closed plans and single metadata publication

**Choice:** The no-std binding family reuses Lattice's bounded workflow-contract
identity type. Requests are capped at 4096 bytes before JSON decoding. Source
identities require nonzero session, pane, and workspace revision plus the exact
recorded producer revision. Only matching lifecycle or focused-pane triggers are
admitted. The sole plan intent is `PublishMetadataReceipt`; no execution port is
available. Each successful host activation calls its receipt publisher once,
without automatic retries after publication failure. Receipt claims are a closed
metadata-only variant and bind the exact plan and supplied activation identity.

**Rationale:** A binding is not an effect grant or a runtime success claim. A
single bounded publication per call is observable without claiming durable
exactly-once delivery or hiding uncertain publication behind retries.

## Risks / Trade-offs

- Cross-repo drift: mitigated by the recorded revision contract.
- Scope creep into an ETL: bounded by the non-goals.
- Over-claim of workflow success: mitigated by the metadata-only receipt boundary.
