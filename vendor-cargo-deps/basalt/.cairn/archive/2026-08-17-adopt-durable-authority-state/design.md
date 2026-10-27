## Context

The vendored UCAN shell exposes `DurableHolderReplayAdmission` and `DurableRevocationChecker`.
Those adapters consume the two pure shared cores at exact Radicle object `a0de79303f3f45c18a34fff535b690a85cb3cf59`.
Basalt's compact-profile shell already accepts generic UCAN replay and revocation traits.

## Decisions

### Decision: Reuse UCAN's published adapters

**Choice:** Basalt composes the UCAN adapters and adds no replay or revocation state transition logic.

**Rationale:** A second consumer should prove reuse, not copy the first consumer's mapping.

### Decision: Depend on both cores directly

**Choice:** Basalt pins both core package names to the canonical Radicle adapter and exact reviewed object.

**Rationale:** Direct declarations make the second consumer and no-fallback source boundary visible to Cargo, Nix, and release checks.

### Decision: Keep ports generic

**Choice:** `BasaltDurableAuthority` is generic over `ReplayLedgerPort` and `RevocationViewPort`.
It owns no Redb database, filesystem path, clock, lock, transaction, or network client.

**Rationale:** Applications retain infrastructure and outage authority.

### Decision: Compose at the shell boundary

**Choice:** The helper returns trait references for `UcanEnforcementBoundaries`.
Basalt policy and profile checks run as before, and UCAN invokes durable replay once at the final commit.

**Rationale:** The pure Basalt authority core must not receive storage or vendor types.

### Decision: Require current revocation evidence

**Choice:** The caller constructs the UCAN durable revocation context with source, minimum generation, and explicit time.
Incomplete, unauthenticated, stale, missing, or unavailable views fail before replay.

**Rationale:** Shared state cannot invent revocation truth or freshness policy.

### Decision: Keep claims bounded

**Choice:** The pilot proves exact-source composition and tested decision mappings only.

**Rationale:** It does not prove global replay exclusion, revocation truth, backend honesty, effect completion, indefinite source availability, or release eligibility.

## Risks / Trade-offs

- The caller must finalize or recover replay reservations after protected effects.
- Different persistence ports can provide different durability observations.
- Caller-supplied authenticity and time observations can be wrong.
- The final UCAN `1.0.0` adapter remains a separate active change.
