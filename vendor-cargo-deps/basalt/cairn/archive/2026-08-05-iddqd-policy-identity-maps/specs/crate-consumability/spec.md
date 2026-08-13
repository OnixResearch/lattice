# Crate Consumability Specification Delta

## ADDED Requirements

### Requirement: Policy identity maps

r[crate_consumability.iddqd_policy_identity_maps.contract]

Basalt MUST validate policy contracts and backend metadata through identity maps whose keys are derived from the mapped values rather than trusted as duplicated external keys.

#### Scenario: Matching policy identity passes

r[crate_consumability.iddqd_policy_identity_maps.fixtures.positive]

- GIVEN a policy artifact whose contract and backend object keys match each value's stable id
- WHEN Basalt builds the runtime policy index
- THEN validation MUST pass and enforcement MUST resolve contracts and backends by those stable ids.

#### Scenario: Duplicate or mismatched identity fails

r[crate_consumability.iddqd_policy_identity_maps.fixtures.negative]

- GIVEN a policy artifact has duplicate ids, an object key that does not match the value id, an empty id, an unsupported backend id, or malformed policy data
- WHEN Basalt builds the runtime policy index
- THEN validation MUST fail closed with deterministic diagnostics before enforcement.

### Requirement: Public policy JSON compatibility

r[crate_consumability.iddqd_policy_identity_maps.serde_boundary]

Basalt MUST preserve the existing Nickel-generated policy JSON object shape while using iddqd-backed identity maps only after parse-time validation.

#### Scenario: Existing generated policy remains compatible

r[crate_consumability.iddqd_policy_identity_maps.docs]

- GIVEN the checked-in generated policy JSON was exported from reviewed Nickel source
- WHEN downstream callers parse it through Basalt's public API
- THEN the public parse and enforcement APIs MUST remain source-compatible while rejecting identity drift earlier.

### Requirement: Final validation

r[crate_consumability.iddqd_policy_identity_maps.final_validation]

The change MUST include positive and negative identity-map coverage plus focused Basalt validation evidence before archive.

#### Scenario: Identity-map fixtures cover drift

r[crate_consumability.iddqd_policy_identity_maps.final_validation.fixtures]

- GIVEN valid and invalid policy identity fixtures
- WHEN focused policy tests and Cairn validation run
- THEN valid fixtures MUST pass and duplicate or mismatched identities MUST fail closed.
