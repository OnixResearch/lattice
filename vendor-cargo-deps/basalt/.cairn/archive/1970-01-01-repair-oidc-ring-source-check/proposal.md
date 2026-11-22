## Why

`checks.oidc-exchange-sources` fails on `264d248`, `1c142e2` and later, and `checks.release`
depends on it, so `nix flake check` fails. Commit `4b09316` moved RS256 verification from
`jsonwebtoken` 11.0.0 to `ring` 0.17.14. It also changed the Nickel source contract to pin
`ring_version` and `ring_checksum` instead of the jsonwebtoken revision and checksum.
The flake check still asserted the removed `jsonwebtoken` upstream revision in
`contracts/oidc-exchange/source-contracts.ncl`.

## What Changes

- Drop the stale jsonwebtoken-revision assertion.
- Assert that the runtime `[dependencies]` pin `ring = "=0.17.14"`, and that the contract
  pins the same version and checksum.
- Assert that the exported source boundary carries that checksum and that the
  `Cargo.lock` entry for `ring` 0.17.14 from crates.io has it.
- Keep the jsonwebtoken assertions, since it remains an exact dev-dependency used to mint
  test tokens. Scope its manifest assertion to `[dev-dependencies]` and fail if it
  reappears in the runtime `[dependencies]`. Its lock checksum assertion is unchanged.
- All other assertions are unchanged.

## Impact

- **Files**: `flake.nix` (`checks.oidc-exchange-sources` only).
- **Testing**: the check passes on the real tree. It fails in scratch copies whose `ring`
  checksum differs in `Cargo.lock` or in the source contract. A full
  `nix flake check` passes.
