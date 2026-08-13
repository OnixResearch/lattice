## Tasks

- [x] [serial] Add the iddqd dependency with deterministic ordered-map usage for policy identity indexes. r[crate_consumability.iddqd_policy_identity_maps.contract]
- [x] [serial] Implement a pure conversion layer that builds contract and backend identity maps while preserving the public policy JSON shape. r[crate_consumability.iddqd_policy_identity_maps.serde_boundary]
- [x] [parallel] Add positive fixtures for valid contract/backend maps with matching keys and value ids. r[crate_consumability.iddqd_policy_identity_maps.fixtures.positive]
- [x] [parallel] Add negative fixtures for duplicate ids, key/value mismatches, empty ids, unsupported backend ids, and malformed policy input. r[crate_consumability.iddqd_policy_identity_maps.fixtures.negative]
- [x] [serial] Update policy parsing diagnostics and public API docs for the identity-map boundary. r[crate_consumability.iddqd_policy_identity_maps.docs]
- [x] [serial] Run focused policy, no-default, CLI-feature, and Cairn validation checks before archive. r[crate_consumability.iddqd_policy_identity_maps.final_validation]
