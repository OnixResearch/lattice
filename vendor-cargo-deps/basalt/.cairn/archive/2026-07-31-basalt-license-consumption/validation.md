# Validation Status

## Result

**IMPLEMENTED AND VALIDATED.** The license-consumption contract records explicit
downstream Basalt adoption evidence with a pure deterministic validator, Nickel
fixtures, and a release-receipt rail.

## Baseline

- The proposal, design, and tasks gates passed.
- Canonical-policy Cairn validation passed (the repo vendored policy is stale;
  draining used the canonical Cairn policy from `../cairn`).

## Implementation Evidence

- `src/license_consumption.rs` and `src/license_consumption_identity.rs` add a
  pure core that validates receipt shape and the AGPL boundary (artifact
  identity, license expression, metadata revision, acknowledged review, and the
  complete non-claim boundary). It performs no I/O.
- `contracts/license-consumption/contracts.ncl` enforces the Nickel receipt and
  fixture shape.
- Positive and negative fixtures under
  `contracts/license-consumption/fixtures/` export to generated JSON:
  - `positive/acknowledged-agpl` passes.
  - `negative/missing-review`, `negative/wrong-artifact-identity`,
    `negative/stale-license-metadata`, `negative/weakened-non-claims`, and
    `negative/wrong-license-expression` fail closed.
  - `malformed/unsupported-schema` is rejected at the Nickel contract boundary.
- The integration test `tests/license_consumption.rs` replays all fixtures.

## Checks Run

- `cargo check --no-default-features` passed.
- `cargo test --test license_consumption --no-default-features` passed (6 tests).
- `cargo test --lib license_consumption --no-default-features` passed (7 tests).
- `cargo clippy --all-targets --no-default-features -- -D warnings` passed.
- `cargo fmt --check` passed.
- The Nix `license-consumption-fixtures` check passed.
- `tools/release-verify.rs` compiled cleanly with the new license-consumption
  freshness and replay steps and receipt.

## Non-claims

The contract records review metadata only. It does not grant an alternate
license, a compatibility promise, a license exception, or legal advice.
