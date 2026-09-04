## Phase 1: Source and contract

- [x] [serial] Record the exact runtime revision, selected source set, command surface, service behavior, authority boundary, and baseline. r[lattice.role_protocol.runtime_profile]
- [x] [serial] Define a closed Nickel contract for immutable runtime, service, capability, state, authority, and non-claim fields. r[lattice.role_protocol.runtime_profile]
- [x] [parallel] Add negative fixtures for stale revision, automatic startup, CLI dispatch claims, widened commands, and unknown fields. r[lattice.role_protocol.runtime_profile]

## Phase 2: Canonical publication

- [x] [serial] Publish the reviewed operator-oneshot profile as Nickel source and canonical JSON. r[lattice.role_protocol.runtime_profile]
- [x] [serial] Publish the canonical profile BLAKE3 identity and selected-runtime source manifest. r[lattice.role_protocol.runtime_profile]
- [x] [serial] Add deterministic Nickel export, JSON comparison, digest, and source-manifest checks. r[lattice.role_protocol.runtime_profile]

## Phase 3: Consumer and closeout

- [x] [parallel] Document the supported OnixOS consumer boundary and unsupported autonomous-dispatch behavior. r[lattice.role_protocol.runtime_profile]
- [x] [serial] Run positive and negative Nickel checks, source-manifest verification, Nix checks, formatting, Cairn validation, and lifecycle gates. r[lattice.role_protocol.runtime_profile]
- [x] [serial] Run the adversarial audit and block archive for widened behavior, secret material, authority transfer, or claim promotion. r[lattice.role_protocol.runtime_profile]
- [ ] [serial] Sync the accepted specification and archive only with an immutable profile publication commit and consumer pin. r[lattice.role_protocol.runtime_profile]
