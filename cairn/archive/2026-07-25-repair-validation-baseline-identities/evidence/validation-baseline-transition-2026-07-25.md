# Validation baseline transition — 2026-07-25

## Octet failing baseline

The checked deny-level run produced:

- Cargo child: exit `101`;
- Octet status: `integration-failure`;
- finding records: `1,141`;
- unique current IDs: `892`;
- accepted/current: `0`;
- new/stale: `892` / `892`;
- old config hash: `b3:3343801154c49d8aaae8bf831619a2c2566b38dfaa8297815c1cb31914a398b8`;
- old profile hash: `b3:65fb9e771347f79a4987efefb092eb7d32cb35dc94beb9b66e51d5a27169b70a`;
- checked baseline file BLAKE3: `ad46b4e8484909c7d4032581085cf626d5efb73f4175921b20cc4e92c4e43d2c`;
- checked baseline set hash: `b3:ff87ad266a61563c76bc4f5835896a080c6d3580a1eb514c1cdf49bda63394be`.

The Cargo exit is expected from explicit deny-level Dylint diagnostics. Octet refuses `--write-baseline` after any unsuccessful Cargo child, so replacing IDs alone cannot repair this state.

## Semantic transition audit

A deterministic ordered comparison covered all `1,141` old and clean-current records. Exactly `1,140` retain identical lint, group, file, line, column, message, crate, and identity status. The sole bounded record transition remains the same `excessive_file_length` finding for `src/config/mod.rs`, whose already accepted source decreased from `676` to `657` lines. After changing observation levels to warning, there are no other non-identity/non-severity mismatches.

The runner-generated warning candidate produced:

- Cargo child: success, exit `0`;
- Octet status: `warning-only`;
- finding records: `1,141`;
- accepted/current unique IDs: `892` / `892`;
- new/stale: `0` / `0`;
- config hash: `b3:a840ccfc25777730b29e23b0f685cf5b8510eb282c760c251f6d57f6d2a2d8f1`;
- profile hash: `b3:907ce8dc5fbef11e97c210005e3e28a20499f7ebe05765dfb8c99e78f5ab6b63`;
- baseline file BLAKE3: `6fc068b868828558f6a37eb6c8d15e3bf9e14a28b340defe1c4ea0523c450c35`;
- baseline set hash: `b3:eed5f5d660b30a9ea345c36515a6e0affbafbf1d038ad19753c1c4dfbb8c8842`.

No lint is disabled or allowed. The canonical wrapper additionally requires successful Cargo classification, exact accepted/current equality, and zero new/stale identities. The Nix negative fixture introduces a new `todo!()` identity in an isolated source copy and must exit `1` with a successful Cargo child and `new_total > 0`.

## Traceability inventory transition

Coverage was generated with the checked repository policy at `cairn-policy/generated/cairn-policy.json`, not a generic Cairn source-root profile. The failing receipt contains:

- requirements: `938`;
- referenced: `926`;
- missing: `12`;
- dangling: `0`;
- missing-set BLAKE3: `c23fc013acdb1e4ae5d093b1f01ddb8c82e7d744ce8f9989f3841ea56e18c457`.

The 12 rows are exactly the accepted Radicle Bounded Exec source, lock, behavior, and outage requirements. The regenerated inventory remains valid with all 12 rows `unassessed`:

- inventory identity: `558f51684014581743a1cf144c5c250603ae181d63ef568b3fcb063c1051b446`;
- report-file BLAKE3: `b9b216b283f845c09a9f4350c595d7c317dcfcf73ae1e551fb8e1c22457b7021`.

Two consecutive generation passes were byte-identical. The inventory does not create Tracey markers, satisfy requirements, or make coverage successful.

## Repository-validation completion digest repair

The already accepted Radicle source cutover changed three byte-bound inputs in the older repository-validation completion fixture while leaving its 11 reviewed decisions and coverage result unchanged:

- `Cargo.toml`: `3d8e7511aa74823762bd852f80bb1b364acd4fc4ea77cfc9f4a259e640a303cd`;
- `README.md`: `a7c10497c17b94ecc643e4820b5d58695ef3b2f3110ae2c8d59aa4ffb2ecac74`;
- historical projection of `tests/test_repository_validation.rs`: `2dbf039bab2daeea5b344e7b81a3ba715fb16353a36fe5d5e335c0e63e271ff1`.

The refreshed reviewed fixture BLAKE3 is `b7f2978944f03879f061a5557e50cb5be28b072fb03b71f71ef8b0374d7bff63`. The exact-byte completion test passes for all selected rows, and its existing classification/path/coverage/marker mutation test remains the negative boundary. No requirement was promoted, reclassified, or removed.

## Portfolio and claim boundary

Replacing IDs under deny configuration was falsified by Cargo exit `101`. Suppression and disabled-lint routes were rejected. Upstream inference of "only accepted Dylint errors" was not selected because it would require a new Octet semantic contract capable of distinguishing all real compiler failures. Warning-level complete observation plus stable-ID admission survived the negative-fixture audit.

This evidence proves only exact transition of the reviewed static finding inventory and deterministic inventory of the current missing requirement set. It does not prove accepted findings correct or harmless, lint completeness, runtime behavior, semantic equivalence, traceability satisfaction, release readiness, or production readiness.
