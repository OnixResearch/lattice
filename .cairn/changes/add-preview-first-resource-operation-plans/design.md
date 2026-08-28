# Design: Preview-first resource operation plans

## Boundary

`lattice-core` owns pure resource-operation planning. The root shell owns catalog calls, approvals, currentness checks, execution, persistence, clocks, and receipts.

The resource-catalog producer owns resource meaning. Lattice accepts only validated producer DTOs through an application-owned adapter.

## Plan model

A plan contains these roles:

- operation type and method revision
- logical resource and optional exact content identity
- selected source and destination replicas
- capability-relative locators as opaque validated facts
- catalog revision and observation identities
- expected entry and byte bounds
- ordered conflict records
- requested resource effects
- required capability and approval classes
- policy, runtime, and credential-reference identities
- explicit non-claims

The plan identity uses domain-separated full-width BLAKE3 framing. Display aliases, raw host paths, sampled fingerprints, and ambient timestamps stay outside authority identity.

## Planning flow

1. The shell loads a validated operation type and instance.
2. A catalog adapter returns bounded resource and replica facts.
3. Pure planning validates roles, currentness inputs, limits, conflicts, capabilities, and non-claims.
4. Planning returns a canonical plan or ordered blockers.
5. The shell persists the plan and requests approval when policy requires it.
6. Execution reloads the exact plan and rechecks catalog revision, observations, policy, authority, and runtime availability.
7. The shell executes only approved effects through injected ports.
8. The shell records bounded post-effect observations and explicit uncertainty.

## Operations

The first contract admits closed operation classes:

- `copy-replica`
- `move-replica`
- `materialize-content`
- `generate-derivative`
- `delete-replica`

`delete-replica` is destructive. It always requires explicit authority and approval. Logical-resource deletion and global content deletion remain outside this change.

## Ports

Application-owned ports provide these capabilities:

- load validated resource facts
- load current replica observations
- obtain an authority decision
- request durable approval
- execute one approved resource effect
- record one bounded outcome

No port exposes database rows, OpenDAL operators, filesystem handles, Iroh objects, or UI types to the core.

## Recovery

A reserved attempt can end as completed, denied, failed, cancelled, or `PendingUnobserved`. Recovery never invents an effect result.

A retry requires a current exact plan and an explicit acknowledgement of duplicate-effect risk when the prior outcome is unknown.

## Verification

Pure tests cover valid plans, role substitution, stale revisions, missing content identity, sampled-fingerprint misuse, conflicts, limits, capability denial, and destructive approval.

Shell tests cover call order, no effect before approval, currentness drift, executor failure, cancellation, uncertain outcomes, and explicit retry.

## Claim boundary

A passing plan proves only deterministic admission over supplied facts. It does not prove catalog truth, source stability, transfer completion, storage durability, external deduplication, or release readiness.
