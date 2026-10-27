# Crate Consumability Delta: Basalt rename

## Requirements

### Requirement: The crate MUST expose `basalt` as the canonical package, library, and CLI identity.

r[crate_consumability.rename_basalt.canonical_name] The crate MUST expose `basalt` as the canonical package, library, and CLI identity.

#### Scenario: Canonical Cargo and CLI names

r[crate_consumability.rename_basalt.canonical_name.cargo_and_cli]

- GIVEN a consumer reads Cargo metadata or enables the CLI feature
- WHEN package, library, and binary names are resolved
- THEN `basalt` is the canonical crate import path and CLI binary name

### Requirement: Documentation and release surfaces MUST use the Basalt identity while preserving former-name discoverability.

r[crate_consumability.rename_basalt.docs_and_release_surfaces] Documentation and release surfaces MUST use the Basalt identity while preserving former-name discoverability.

#### Scenario: Basalt docs and verifier names

r[crate_consumability.rename_basalt.docs_and_release_surfaces.examples]

- GIVEN a maintainer or consumer follows README examples or release-verifier commands
- WHEN they copy the documented commands
- THEN the commands use `basalt` names and explain that Basalt was formerly `ucan-nickel-contracts`

### Requirement: The rename MUST classify intentional old-name references so compatibility is reviewable.

r[crate_consumability.rename_basalt.compatibility_classification] The rename MUST classify intentional old-name references so compatibility is reviewable.

#### Scenario: Stable policy schema compatibility

r[crate_consumability.rename_basalt.compatibility_classification.policy_schema]

- GIVEN stale-name searches run after the rename
- WHEN `ucan-nickel-contracts` remains in source-controlled files
- THEN each remaining occurrence is either historical Cairn archive content, former-name discoverability text, or an intentionally stable policy schema identifier

### Requirement: Rename verification MUST prove the renamed crate still passes consumer and release gates.

r[crate_consumability.rename_basalt.rename_verification] Rename verification MUST prove the renamed crate still passes consumer and release gates.

#### Scenario: Rename gate suite

r[crate_consumability.rename_basalt.rename_verification.gates]

- GIVEN the Basalt rename is implemented
- WHEN formatting, tests, clippy, policy freshness, Nix flake checks, Cairn validation, and stale-name classification run
- THEN the renamed crate is ready to commit without unreviewed stale identity references
