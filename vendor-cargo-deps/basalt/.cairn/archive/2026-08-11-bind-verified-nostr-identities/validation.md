# Validation

## Producer boundary

Basalt vendors UCAN revision `277ab768eb532ac0c715a3bbff2a7dc37a702f11`.
Source parity and the 677-file BLAKE3 snapshot check passed.

The public adapter accepts only the `VerifiedNostrCredential` contract from revision `c9d43a7c11dd2fab4d5cb3d100012fa9a2e5b026`.

## Producer identities

- Schema BLAKE3: `45824d99ae765e885f085ef2c8c8294343415af8e5489c35509c3e255476e258`
- Implementation-profile BLAKE3: `ced0ffcfe006a48ffd8f2b856ad0da9641a03a24e6ebda3877163c398252897c`
- Policy BLAKE3: `9f5df806b5fac55a35a63f58c86e4218de1414141c20c999cbd140ea3c0556a2`

## Checks

- Nickel policy typecheck and generated policy freshness passed.
- No-default tests passed, including 86 library tests and all integration tests.
- CLI-feature tests passed, including 93 library tests and all integration tests.
- Strict Clippy passed for all targets and features.
- The release verifier passed 170 no-default tests, 183 CLI tests, package inventory, Clankers downstream checks, documentation, and all focused rails.
- `nix flake check path:$PWD -L` passed. The unavailable remote builder message was non-blocking because local builds completed.
- Native Cairn validation and proposal, design, and task gates passed with no issues.
- Focused native traceability passed with 10 of 10 requirement references under the current `trellis-rust` source layout.

## Cairn policy note

The repository policy projection predates the current native Cairn schema and lacks `schemas.*.output_path`.
Native lifecycle checks used the current reviewed Cairn default policy with policy hash `2c0b44345b9d2e2b3eff5fa7db7a14b9681c6bfcf7a4bb651f13bcadc1f91757`.
The default current traceability profile targets `crates/`, while Basalt uses `src/` and `tests/`.
The focused traceability run used the current `trellis-rust` source layout and a temporary requirement projection.
Repository Nix checks also passed with the pinned repository inputs.

## Non-claims

These checks do not prove a human identity, producer trust, relay completeness, replay-store durability, authorization, deployment, or release eligibility outside the recorded checks.
BLAKE3 receipt identities detect changed bytes under the declared algorithm. They do not authenticate a producer.
