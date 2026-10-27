# Tasks

## Implementation

- [ ] [serial] Extract native gateway transport DTOs and exact request canonicalization into the producer-owned standalone contract; migrate the native receiver and API. r[mantle.native_consumer_observations.signed_gateway]
- [ ] [serial] Sign native response snapshots with the existing completion key and retain independent terminal completion verification. r[mantle.native_consumer_observations.signed_gateway]
- [ ] [parallel] Extend producer consumer-verifier with capability-rooted typed source-tree remeasurement and bounded source-relative inventory, preserving file closures and identities. r[mantle.native_consumer_observations.source_tree]
- [ ] [serial] Migrate the complete Lattice consumer to the real signed native gateway and exact typed bundle bindings. r[mantle.native_consumer_observations.signed_gateway] r[mantle.native_consumer_observations.source_tree]

## Verification and claims

- [ ] [serial] Integration owner exercises complete source-tree and legacy file bundles and negative drift/shape/path/membership cases. r[mantle.native_consumer_observations.source_tree]
- [ ] [serial] Integration owner exercises native signed asynchronous submission, restart, duplicate/gap events, result verification, owner/fence failures and teardown-confirmed cancellation. r[mantle.native_consumer_observations.signed_gateway]
- [ ] [serial] Integration owner records exact local source acquisition and explicit unpublished-source/non-production readiness limits; no remote mutation, acceptance or archive. r[mantle.native_consumer_observations.signed_gateway]

## Local integration correction freeze

The local correction commit following `cc1f0d2b14fbea1c2753dd8e52d965bd38b047d8`
preserves the signed gateway dispatch under one authenticated authorization and
the source-tree verifier work already present in the imported ancestor
`644511326ea8efc2f37c3526dca0f0d13d2bdc88`. It retains the current first-party
formatting corrections, removes the unused journal `Completion` import, and
collapses the equivalent gateway, gateway-test and source-tree selection
branches required by strict Clippy. It does not change signed transport,
terminal completion or bundle identity encodings, or disable a verifier path.

At this local freeze, the integration owner's strict Clippy and focused gateway
test run (`bg_213`) is pending; this note does not claim it passed. Consumer
refresh, executable identity migration, complete application build and runtime
qualification remain the integration owner's work. The unchecked tasks above
remain unaccepted. This source is local and unpublished: no remote mutation,
lifecycle acceptance/archive, or deployment activation is authorized or claimed.
