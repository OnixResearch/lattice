# Orchestration identities

<!-- r[impl lattice.agent_orchestration.nominal_identities.inventory] -->
<!-- r[impl lattice.agent_orchestration.nominal_identities.inventory.display] -->
<!-- r[impl lattice.agent_orchestration.nominal_identities.docs] -->

Lattice gives each admitted orchestration identity one Rust role. This prevents a value from satisfying an unrelated core API only because both values use text.

## Semantic inventory

| Class | Rust types | Meaning |
|---|---|---|
| Stable text identity | `DefinitionId`, `TriggerId` | Definition and configured trigger names. |
| Idempotency input | `TriggerIdempotencyKey` | One untrusted trigger-delivery key before scope binding. |
| Stable decision identity | `ReservationId` | One budget reservation, including the named retry preflight reservation. |
| Revision digest | `DefinitionRevision`, `WorkflowRevision` | Exact BLAKE3 identities for admitted definition and workflow bytes. |
| Decision digest | `InstanceId`, `DispatchId`, `ProposalId`, `ApprovalId`, `FactId` | Distinct state-machine roles with separate hash domains. |
| Policy digest | `AuthorityPolicyIdentity` | Exact authority-policy projection used by an approval. |
| Trigger digest | `TriggerScopedKey` | Exact definition, trigger, and idempotency binding. |
| Event digest | `EventId`, `PreviousEventId`, `LedgerHeadId`, `StateDigest` | Distinct append, prior-link, current-head, and resulting-state roles. |
| Existing numeric identity | `RunId` | Durable run allocation and ownership. This contract is unchanged. |
| External wire text | Nickel, CLI, webhook, scheduler, JSON, and redb strings | Compatibility values that must pass a checked adapter. |
| Display text | Objectives, labels, summaries, actors, diagnostics, and redacted output | Human-readable content. These values are not nominal identities. |
| Deferred domains | Provider, credential, and external protocol values | Values owned by other authority or transport boundaries. |

Text constructors reject empty values, values larger than 256 bytes, surrounding whitespace, and control characters. Digest constructors accept only 64 lowercase hexadecimal BLAKE3 characters.

## Core and adapter boundary

`src/orchestration/identity.rs` owns private storage, checked constructors, explicit accessors, and transparent string serialization. `src/orchestration/core.rs` keeps typed keys and values through transitions, budget decisions, recovery correlation, and replay.

Adapters preserve the existing external shape:

- `src/orchestration/format.rs` uses string DTOs for Nickel definitions and proposals. It admits each identity before core construction.
- `src/orchestration/shell.rs` admits CLI, scheduler, and webhook text before loading state or making a transition.
- `src/store/mod.rs` converts typed keys to existing redb text keys. It checks text from redb before core use.
- JSON reports serialize identity wrappers as their previous string values.
- `RunOwner::Orchestration` stores typed instance and dispatch owners. Its JSON projection remains two strings.

These adapters do not infer identities. A malformed or wrong-role value fails before a workflow, provider, credential, scheduler, or webhook effect starts.

## Compatibility

The migration does not change the Nickel contracts, JSON field names, redb table names, text keys, event hash inputs, or event ordering. Existing supported records must deserialize through checked constructors and replay without effect execution.

Compile-pass and compile-fail fixtures prove same-role acceptance and cross-role rejection. Persistence, crash recovery, replay, tamper, scheduler, webhook, and inspection tests cover the runtime compatibility boundary.

## Claim boundary

Typed identities prove local role separation and checked syntax for the tested code. They do not prove provider truth, remote cancellation, external exactly-once effects, independently witnessed history, or production readiness.
