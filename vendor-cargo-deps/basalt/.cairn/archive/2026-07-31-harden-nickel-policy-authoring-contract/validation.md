# Validation Status

## Result

**IMPLEMENTED AND VALIDATED.** Basalt's Nickel policy authoring contract and
projection parity are hardened. Reviewed source is rejected early, external
projections are rejected by Rust authoritatively, and the generated artifact
stays meaning-compatible with the Rust consumer.

## Implementation Evidence

- `contracts/policy-prelude.ncl` provides shared hardened contracts and
  predicates: exact schema, normalized ids, supported backend ids, non-empty
  unique allowlists, map-key/id alignment, and lexical resource/ability values.
- `contracts/policy.ncl` uses the prelude to validate the whole source policy:
  exact schema, at least one contract, key/id alignment, unique identities,
  supported backend metadata, non-empty unique allowlists, and lexical shapes.
  The exported `contracts/generated/policy.json` remains byte-identical.
- `src/policy_projection.rs` is a pure deterministic core that validates a
  loaded `Policy` against the supported Rust contract (schema, backends,
  identities, allowlists, lexical values). It performs no I/O and no Nickel
  evaluation.
- Positive/negative Nickel fixtures under `contracts/policy-authoring/fixtures/`
  prove authoring behavior. External-JSON fixtures prove `parse_policy_json`
  and `validate_projection` fail closed when Nickel is bypassed.
- `tests/policy_projection.rs` replays the external JSON cases and the generated
  artifact. The release verifier and flake include the projection parity rail.
- The active iddqd runtime identity-map check remains authoritative for all
  externally supplied JSON; this package owns only source shape and projection
  parity. This regression repair also refreshed the Basalt policy BLAKE3
  identity in `ucan_semantics_identity`, the authority-core fixture base, and
  regenerated authority-core/ucan-semantics fixtures after the onixos policy
  extension changed the generated artifact identity.

## Checks Run

- `cargo check --no-default-features --all-targets` passed.
- Full no-default and CLI-feature test suites passed (all integration and unit
  targets green).
- `cargo clippy --all-targets` passed for both `--no-default-features` and the
  `cli` feature.
- `cargo fmt --check` passed.
- Nix `policy-export` and `policy-authoring-fixtures` checks passed.
- `tools/release-verify.rs` compiled cleanly with the projection parity step.

## Non-claims

Nickel policy conformance and projection parity do not authorize requests,
validate UCAN tokens, establish backend truth, or replace Rust receipt
authority.
