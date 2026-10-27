## Context

Basalt has Nickel policy, a pure enforcement core, UCAN-bound authority decisions, and receipt infrastructure. Consumers currently map runtime effects into product-specific contract names and abilities.

The selected agentOS permission model is a design reference for distinct scopes and pre-effect denial. Basalt will not adopt agentOS defaults, glob semantics, virtual kernel, V8 runtime, or sandbox claims.

## Decisions

### Decision: Basalt owns portable policy meaning only

**Choice:** Define closed scopes for filesystem, network, process, environment, and binding effects. Consumers convert native requests into normalized effect subjects and retain all parser, host, and effect authority.

**Rationale:** Basalt can decide policy consistently without absorbing filesystem, network, process, or runtime implementation.

### Decision: Default deny and reject ambiguity

**Choice:** Every profile has an explicit default, with deny as the required secure baseline. Rules use declared exact or prefix match modes and explicit priorities. Equal-priority conflicting matches deny as ambiguous.

**Rationale:** Ordered last-match behavior can widen authority after harmless-looking rule reordering.

### Decision: Approval and permission are separate

**Choice:** The effect permission core returns allow or deny. Human approval remains a separate consumer-owned gate and cannot override a Basalt denial or missing UCAN authority.

**Rationale:** Interactive approval is workflow state, not kernel or policy enforcement.

### Decision: Decisions require current consumer enforcement

**Choice:** Receipts bind policy and request identity, but consumers must recheck current policy, capability, adapter, and subject facts immediately before effects.

**Rationale:** A receipt is not a live capability or proof that enforcement happened.

## Risks / Trade-offs

- Portable subjects can lose platform-specific meaning. Consumer mapping fixtures must reject lossy conversion.
- Prefix rules can be misused as path confinement. Documentation must require Cap Root and capability-relative I/O where applicable.
- Cross-repository conformance can drift. Fixtures must pin exact consumer cohorts.
- The agentOS source must be pinned to an exact reviewed revision before implementation.
