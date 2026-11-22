## Phase 1: Transport

- [x] [serial] Make the serve loop generic over `Listener` and keep the Unix path unchanged; add `exchange_connected`, r[lattice.workflow_interchange.guest_listener]. Evidence: `evidence/transport-tests.log` (35 passed, including the 6 unchanged `socket` tests), `evidence/lib-exchange-tests.log` (18 passed).
- [x] [serial] Add the C5 vsock profile and the guest vsock listener on nix `VsockAddr`, with no new dependency and no `unsafe`, r[lattice.workflow_interchange.vsock_profile] and r[lattice.workflow_interchange.vsock_listener]. Evidence: `evidence/transport-tests.log` (`reconnect::vsock_profile_*`, `listener::vsock_serving_*`). Real `AF_VSOCK` is not exercised.
- [x] [serial] Add the C6 rule table and the C7 continuity check as pure functions, with generated, exhaustive fixtures and fixture tests, r[lattice.workflow_interchange.reconnect_rules] and r[lattice.workflow_interchange.observation_continuity]. Evidence: `evidence/transport-tests.log` (`exchange-recovery-fixtures` 3 passed, `continuity-fixtures` 1 passed).

## Phase 2: Application

- [x] [serial] Add handler profile v2 with listener selection, a separate identity domain, refused pause and resume, Nickel contracts, and positive and negative fixtures, r[lattice.workflow_interchange.handler_profile_v2]. Evidence: `evidence/lib-exchange-tests.log` (`handler_profile_v2_*`, `handler_profile_versions_keep_separate_identity_domains`, `typed_handler_profile_v2_loads_and_negative_profiles_fail`).
- [x] [serial] Drive the real `ApplicationHandler` through a fault-injecting `Listener` over Unix socketpairs for every planned S2 scenario, r[lattice.workflow_interchange.reconnect_rules.scenario.lost-dispatch], r[lattice.workflow_interchange.reconnect_rules.scenario.not-accepted], r[lattice.workflow_interchange.reconnect_rules.scenario.cancel-race], r[lattice.workflow_interchange.observation_continuity.scenario.restart] and r[lattice.workflow_interchange.observation_continuity.scenario.gap]. Evidence: `evidence/reconnect-tests.log` (10 passed; three earlier consecutive runs also passed).

## Phase 3: Documentation and checks

- [x] [serial] Document the guest gateway boundary and update the transport documentation. Evidence: `docs/workflow-exchange-guest-gateway.md`, `docs/workflow-exchange-transport.md`.
- [x] [serial] Run focused tests, clippy `-D warnings` on the touched packages, and `octet-precommit-check`, and record the evidence. Evidence: `evidence/transport-tests.log`, `evidence/lib-exchange-tests.log`, `evidence/reconnect-tests.log`, `evidence/transport-clippy.log`, `evidence/lattice-clippy.log`, `evidence/transport-octet-status.json` (deny-all clean, 0 findings), and `evidence/octet-precommit.log` (lattice 1052/1052 accepted, 0 new, 0 stale; lattice-core 25/25, 0 new, 0 stale). Repository gates (`scripts/check.rs`, `nix flake check`) belong to the integration owner.

## Not in this change

- Real `AF_VSOCK` qualification. Kiln's Cloud Hypervisor test (Q2) owns it.
- The Iroh host gateway, principal and placement binding (Kiln #54), and every cross-host, live-model and production claim.
