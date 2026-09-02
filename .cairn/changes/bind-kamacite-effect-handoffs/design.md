## Context

Lattice already validates a complete Choregraph intent and Basalt authority facts before it persists an attempt. The operation field remains a runtime-local value.

The change adds semantic identity only at declared product effect boundaries. Internal graph values keep their existing owners and representations.

## Success Contract

One selected effect retains the same exact Kamacite semantic operation identity across registry resolution, intent binding, Basalt decision, durable attempt, adapter invocation, and terminal observation.

Any mismatch stops before product dispatch. Prior receipts and replay records never become fresh authority.

## Decisions

### 1. Resolve through an explicit registry

A reviewed registry maps complete admitted Choregraph operation facts to an exact Kamacite descriptor identity.

The mapping cannot derive identity from a name, schema shape, payload hash, or nearby descriptor. Missing and ambiguous entries deny dispatch.

### 2. Keep Kamacite types at the inbound boundary

A narrow adapter validates the descriptor and returns a Lattice-owned `SemanticOperationRef`.

Generic Kamacite or Preserves values do not enter `lattice-core`, persistence models, or product adapter traits.

### 3. Extend exact authority binding

The existing Choregraph effect authority input gains the semantic operation identity. Basalt policy and decision inputs must bind that same identity.

A Basalt version without the compatible semantic identity contract cannot satisfy live integration readiness.

### 4. Persist before dispatch

The durable attempt stores the semantic identity and any admitted compatibility identity before adapter invocation.

Terminal success, failure, or unknown observations retain those identities. They do not claim handler correctness or external effect truth.

### 5. Keep compatibility directional

A different target identity requires exact source, target, direction, and execution context through Kamacite compatibility.

Replay-only compatibility cannot admit live dispatch. Reverse and stale artifacts fail closed.

### 6. Preserve replay denial

Replay reconstructs recorded semantic identities and compatibility facts. It does not call the registry for a new identity or invoke product adapters.

### 7. Start with two bounded effect families

The first cohort covers one process effect and one Animus agent effect. Other effect families remain unchanged and unsupported by this profile.

## Risks and Controls

- Registry drift can stop valid workflows. Exact source revision, descriptor identity, and migration fixtures make changes explicit.
- Basalt coordination can lag. The plan records this as a hard implementation dependency rather than adding a local fallback.
- Persisted records can mix identity versions. Versioned records and negative recovery fixtures reject mixed attempts.
- Identity can be mistaken for capability. Existing authority and currentness gates remain required.

## Non-Claims

This design does not prove operation behavior, handler correctness, provider truth, process safety, authorization, external success, exactly-once execution, or release eligibility.
