## Why

Lattice binds Choregraph intent, Basalt policy, durable attempts, and product adapters before effects. The chain uses Lattice and Choregraph identities, but not one canonical semantic operation identity.

Molten can carry an exact Kamacite operation identity through manifests, handlers, logs, and replay. Lattice can therefore schedule the right local label while targeting a changed operation contract.

Lattice needs one exact semantic identity across handoff, authorization, durable attempt, adapter dispatch, and observation.

## What Changes

- Pin one reviewed Kamacite semantic operation and compatibility cohort.
- Add a narrow registry that maps admitted Choregraph operation facts to exact Kamacite `effect-operation` identities.
- Require the same identity in Basalt authorization inputs and decisions.
- Persist the identity before product dispatch and retain it in terminal observations.
- Permit substitution only through exact directional compatibility admitted for the requested context.
- Preserve replay as reconstruction only. Historical identities cannot grant fresh dispatch.
- Add end-to-end positive and negative fixtures for one process effect and one agent effect.

## Dependencies

- Kamacite owns semantic operation descriptors and directional compatibility artifacts.
- Choregraph owns graph and machine decisions. It does not authorize effects.
- Basalt must publish a compatible runtime-effect semantic identity contract before live authorization integration completes.
- Lattice retains policy loading, currentness, durable attempts, adapters, execution, persistence, and recovery.

## Non-Goals

- Replacing Choregraph machine identity, Lattice operation contracts, Basalt policy, or product adapters.
- Deriving semantic identity from operation names, schemas, or payloads at runtime.
- Treating a matching identity as authorization, capability, execution, or success evidence.
- Adding Kamacite to internal workflow values that never cross a declared effect boundary.

## Impact

- **Core**: Lattice-owned nominal semantic operation references enter pure binding and attempt decisions.
- **Adapters**: a narrow registry and Kamacite boundary validate descriptors without exposing generic values inward.
- **Authority**: Basalt requests and decisions bind the same semantic identity as the Choregraph handoff.
- **Persistence**: durable attempts and terminal observations retain the exact identity and compatibility context.
- **Testing**: stale, name-equal, wrong-direction, wrong-context, and adapter-mismatch cases stop before effects.
