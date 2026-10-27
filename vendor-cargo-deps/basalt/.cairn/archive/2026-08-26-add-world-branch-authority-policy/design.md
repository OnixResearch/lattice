## Context

Basalt already owns reviewed Nickel policy shape, deterministic matching, UCAN-bound decisions, runtime-effect permission profiles, and metadata-only receipts.

World branching introduces a distinct policy question. A consumer must decide how an existing capability relates to a new branch and which obligations precede activation.

Basalt can own this portable policy meaning. It cannot own product branch state or capability realization.

## Decisions

### Decision: Define a closed branch-authority vocabulary

**Choice:** Add closed branch actions and result modes for copyable, attenuated, linear, simulation-only, promotion-gated, replace-before-activation, and non-branchable authority.

Unknown values make a profile invalid.

**Rationale:** Closed values prevent silent fallback to a wider copy rule.

### Decision: Normalize policy inputs without native secrets

**Choice:** The request binds policy identity, capability class and safe identity, source and destination scopes, branch action, requested limits, currentness references, and optional verified UCAN facts.

It excludes bearer tokens, private keys, credentials, raw host paths, private policy bodies, and secret entropy.

**Rationale:** Basalt needs behavior-relevant policy facts, not ambient access to consumer secrets.

### Decision: Return obligations, not effects

**Choice:** A passing decision returns one closed mode and explicit obligations. Obligations can require independent grant proof, attenuation limits, durable transfer, simulation adapter identity, promotion evidence, or replacement before activation.

Basalt performs none of those operations.

**Rationale:** A pure decision cannot prove that a token was minted, ownership moved, or an adapter enforced the result.

### Decision: Deny ambiguity and authority widening

**Choice:** Profiles deny by default. Equal-priority conflicting rules, overlapping incompatible obligations, missing required facts, wider destination limits, and unsupported scope mappings deny.

**Rationale:** Rule order and lossy mapping must not widen authority.

### Decision: Treat linear mode as a stateful consumer obligation

**Choice:** A linear decision requires exact source, destination, capability, expected generation, and durable-currentness facts. The receipt states that the consumer must prove source deactivation before destination activation.

Basalt never reports transfer completion.

**Rationale:** Exclusive ownership requires durable product state and recovery behavior outside the policy core.

### Decision: Keep simulation and live use disjoint

**Choice:** A simulation-only decision names required simulation behavior and forbids live fallback. A promotion-gated decision names the required promotion class but does not prove promotion.

**Rationale:** Branch exploration must not leak effects through an available live adapter.

### Decision: Recheck currentness at consumer boundaries

**Choice:** Decisions bind supplied policy, UCAN, revocation, replay, scope, and generation observations. Consumers must reevaluate at activation, transfer, promotion, and effect dispatch.

**Rationale:** A prior passing receipt is not a durable capability.

### Decision: Keep the implementation pure

**Choice:** Nickel owns authoring and schema contracts. Generated JSON enters the Rust pure core through caller-owned loading.

The core performs validation, matching, obligation construction, and safe receipt projection only.

**Rationale:** This preserves Basalt functional-core and imperative-shell boundaries.

## Rollout

1. Add Nickel contracts and generated-policy freshness checks.
2. Add pure DTOs, validation, matching, and obligation decisions.
3. Add simulation-only and non-branchable fixtures.
4. Add attenuation, linear, replacement, and promotion-gated fixtures.
5. Add frozen Molten consumer mappings and overclaim checks.

## Risks / Trade-offs

- Portable capability classes can hide native differences. Consumers must reject lossy mappings.
- Linear decisions can be mistaken for transfer completion. Receipts need explicit non-claims.
- Simulation profiles can drift from live adapters. Basalt proves only policy selection over supplied facts.
- More modes increase policy review cost. Closed contracts and negative fixtures keep behavior explicit.
