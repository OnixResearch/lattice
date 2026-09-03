# Proposal: Pilot linear process-permit flow

## Why

Lattice owns effect execution, durable attempts, persistence, retries, and uncertain outcomes. Choregraph owns static graph and effect-intent admission.

The runtime does not yet consume a proof-linked linear resource-flow contract. A process pilot can validate the complete boundary without broad workflow migration.

The pilot uses a single-use symbolic process permit. It does not replace Basalt authorization, scheduler capacity, Bounded Exec ownership, or operating-system handles.

## What Changes

- Pin the published Choregraph resource-flow handoff and its Trellis and Kamacite identities.
- Add an opt-in process-permit profile for local shell-node execution.
- Model `Available`, `Owned`, `Settled`, and `Unknown` permit states in the pure Lattice core.
- Bind one permit occurrence to one Choregraph effect intent and one durable Lattice attempt.
- Require product authorization and durable preparation before Bounded Exec starts a child.
- Record `Settled` only after a trustworthy terminal observation.
- Record `Unknown` when execution lacks a trustworthy terminal observation.
- Deny automatic retry, permit restoration, or occurrence reuse from `Unknown`.
- Require a new permit identity and explicit duplicate-risk acknowledgment for any fresh retry.
- Add positive, negative, crash-window, recovery, compatibility, and evidence tests.

## Outcome and Durable Capability

The immediate outcome is one end-to-end consumer of the new Choregraph resource-flow profile.

The durable capability is single-use process-attempt ownership with explicit uncertainty and no silent permit restoration.

Lattice maintains runtime state, authorization integration, persistence, execution, observations, and recovery. It does not maintain generic row unification.

## Impact

The change affects pure runtime admission, shell orchestration, persistence schemas, local-process tests, Choregraph intake, operator inspection, and evidence.

Legacy workflows remain unchanged unless they select the pilot profile.

## Dependencies

- Choregraph change `admit-linear-resource-flow`
- Its exact Trellis and Kamacite producer profiles
- Existing Basalt authorization integration
- Existing pinned Bounded Exec revision
- Workspace coordinator `adopt-linear-resource-flow-contracts`

Implementation remains blocked until Choregraph publishes an immutable compatible handoff.

## Non-goals

- Do not adopt Mirth source, compiler, language, or runtime.
- Do not add a generic resource-row checker to Lattice.
- Do not treat a process permit as a credential, capability token, scheduler slot, or operating-system handle.
- Do not claim sandboxing, process correctness, external cancellation, exactly-once execution, or release readiness.
- Do not migrate file, network, agent, provider, or remote effects in this pilot.
- Do not restore or reuse an unknown permit automatically.
