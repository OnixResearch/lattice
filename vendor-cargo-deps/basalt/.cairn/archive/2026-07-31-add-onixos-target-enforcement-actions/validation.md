# Validation Status

## Result

**IMPLEMENTED AND VALIDATED.** The closed OnixOS target action set is added to
the raw UCAN enforcement adapter with reviewed policy mappings, an identity-bound
`lib.ucanSource` export, a positive mapping matrix, and negative substitution
tests.

## Implementation Evidence

- `EnforcementAction` gains six closed variants with exact contract, ability,
  and resource-prefix mappings. There is no generic action constructor.
- `bind_enforcement_action` now admits only resources inside the reviewed
  prefix and denies action, contract, ability, and resource-prefix substitution
  before replay. State-sync publish does not attenuate delete or admin.
- `contracts/policy.ncl` adds one reviewed Nickel contract for each target and
  state-sync action; `contracts/generated/policy.json` is regenerated.
- The flake exports `lib.ucanSource = ./vendor/ucan`, naming the checked-in
  reviewed snapshot for downstream packaging.
- Integration tests in `tests/ucan_enforcement/onixos_targets.rs` cover the
  mapping matrix and substitution/attenuation negative cases.

## Checks Run

- `cargo check --no-default-features --all-targets` passed.
- `cargo test --test ucan_enforcement --no-default-features` passed (22 tests,
  including 3 new onixos-target tests).
- `cargo test --lib ucan_enforcement --no-default-features` passed (9 tests).
- `cargo clippy --all-targets` passed for both `--no-default-features` and the
  `cli` feature.
- `cargo fmt --check` passed.
- The Nix `policy-export` and `downstream-compat-config` checks passed.
- `basalt.lib.<system>.ucanSource` evaluates to the vendored UCAN tree.

## Non-claims

A live result proves only bounded evaluation for the exact action, contract,
resource, ability, request, and supplied verification boundaries. It does not
prove machine identity discovery, global revocation freshness, replay
durability, transport security, effect execution, effect correctness, or
release eligibility.
