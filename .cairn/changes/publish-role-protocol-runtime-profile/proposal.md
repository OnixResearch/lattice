## Why

The archived projected-role runtime is implemented and tested, but it does not publish one machine-readable runtime and service-integration profile. OnixOS cannot bind native choreography roles without inventing package, invocation, lifecycle, state, authority, or capability facts.

## What Changes

- Publish a closed Nickel contract for one projected-role runtime profile.
- Publish one concrete profile bound to Lattice runtime revision `cbb5db0e4844b466dcd1877218eb1f00a2a7caf2`.
- Bind the package, binary, command prefix, supported operator commands, state backend, service mode, authority boundary, and consumer-required inputs.
- State explicitly that the reviewed CLI profile is operator-invoked, does not start at boot, and cannot dispatch protocol effects.
- Publish canonical JSON, its BLAKE3 identity, and a BLAKE3 manifest for the selected runtime source set.
- Add positive, negative, source-manifest, canonical-export, and lifecycle checks.

## Success Criteria

- A consumer can select the exact profile without guessing runtime or service behavior.
- The profile identity changes when any semantic profile field changes.
- The source manifest verifies against the selected immutable runtime revision.
- Unknown fields, stale revisions, widened commands, automatic start, or claimed CLI dispatch fail validation.
- Profile admission does not grant runtime, adapter, credential, transport, or deployment authority.

## False Completion

A commit hash without a typed profile is not completion. A package name without lifecycle and authority constraints is not a service-integration profile.

A profile that claims the current CLI can dispatch protocol effects is false. The reviewed CLI uses unavailable authority and no configured adapters.

## Impact

- **Contracts**: one closed Nickel runtime-profile contract.
- **Profiles**: one canonical operator-oneshot profile plus JSON and BLAKE3 identity.
- **Evidence**: one selected-source manifest tied to the immutable runtime revision.
- **Checks**: positive and negative Nickel evaluation plus deterministic export and manifest checks.
- **Consumers**: OnixOS can pin the profile while retaining placement and deployment authority.

## Dependencies

- The archived `execute-projected-role-protocols` change at `cbb5db0e4844b466dcd1877218eb1f00a2a7caf2` remains the runtime implementation authority.
- The archived Choregraph producer remains pinned at `0b2000edaf86d5a7a8601246afbc0d526f4a2a07`.
- Existing Lattice effect-authority and adapter boundaries remain authoritative.

## Out of Scope

- Adding a daemon, automatic dispatch, boot activation, or network listener.
- Adding new protocol semantics, adapters, credentials, or authorization.
- Claiming deployment, transport delivery, external-role correctness, protocol completion, production readiness, or release eligibility.

## Affected Specs

- `role-protocol-execution`: immutable runtime and service-integration profile publication.
