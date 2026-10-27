## Context

`bind-ucan-protocol-profiles` introduces exact profile cohorts and a Basalt-owned verified UCAN authority input. This change adds final UCAN `1.0.0` only after the UCAN repository publishes the required profile implementation and evidence.

The final profile carries one exact Invocation command, arguments, subject, executor, ordered Delegation proof path, policy result, nonce, time bounds, and public token identities. Basalt must map these facts to application policy without widening them.

## Decisions

### Decision: Require exact archived upstream evidence

**Choice:** The vendored snapshot refresh must bind exact archived UCAN wire and authorization receipts, the final profile ID, UCAN support-catalog BLAKE3, source revision, file manifest, and preserved licenses.

Failed, missing, stale, unsupported, or untested upstream evidence blocks adoption.

**Rationale:** Workspace proximity and source presence do not establish interoperability.

### Decision: Select a separate final-profile adapter

**Choice:** A visible composition root selects either the compact or final adapter from an admitted typed cohort. The final adapter accepts only UCAN's live verified final-profile Invocation value.

No adapter accepts raw bytes from another profile or guesses a decoder.

**Rationale:** Nominal adapter separation prevents profile confusion.

### Decision: Map only complete authority facts

**Choice:** The adapter projects profile, source cohort, Invocation and proof identities, subject, executor, command, arguments identity, policy disposition, request binding, revocation class, replay class, and non-claims into Basalt's application input.

If a Basalt action requires an external resource fact, the caller supplies a separately verified owner binding. Missing or lossy facts deny.

**Rationale:** An application mapping must not invent resource meaning absent from UCAN's bounded result.

### Decision: Keep command-to-action policy in Nickel

**Choice:** Reviewed Nickel policy defines a closed finite mapping from final-profile commands and required subject/resource classes to exact `EnforcementAction`, contract, ability, and operation-profile values.

The Rust core consumes the exported typed mapping. Unknown commands, duplicate mappings, ambiguous priorities, and incomplete required facts deny.

**Rationale:** Basalt owns application policy, while UCAN owns protocol command hierarchy.

### Decision: Complete policy work before state consumption

**Choice:** UCAN and Basalt must preserve the existing rule that all fallible mapping, policy, allocation, and receipt preparation completes before the final replay commit. The adapter does not call a second replay boundary.

**Rationale:** Cross-component composition must not spend replay state before a possible Basalt denial.

### Decision: Keep live and persisted authority separate

**Choice:** A successful final-profile path returns the existing private Basalt live capability bound to the borrowed exact request. Profile-bound receipts remain redacted, serde evidence with no live constructor.

**Rationale:** Persisted compatibility evidence cannot become reusable authorization.

## Flow

```text
explicit final profile cohort
  -> UCAN live verified Invocation
  -> final-profile Basalt adapter
  -> complete Basalt-owned authority input
  -> Nickel-exported closed action mapping
  -> pure Basalt authority decision
  -> existing final replay composition
  -> live Basalt capability + redacted profile receipt
```

## Risks and trade-offs

- Final-profile arguments may contain sensitive or large IPLD values. Basalt receipts use bounded identities, not payload copies.
- Mapping one command to an action can be unsafe without complete subject or external resource facts. Incomplete mappings deny.
- Supporting compact and final profiles creates migration cost. Removing compact support needs a separate change.
- Upstream evidence can establish bounded implementation facts only. It does not prove key trust, external ownership, replay durability, or effect correctness.
