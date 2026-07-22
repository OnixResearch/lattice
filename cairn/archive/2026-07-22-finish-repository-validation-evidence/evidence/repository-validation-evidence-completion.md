# Repository-validation evidence completion

## Result

All 11 reviewed repository-validation rows now have exact maintained evidence.

- A configured dev path dependency is compiled and called by `tests/test_repository_validation.rs`.
- Its nested `src/coverage/result.rs` remains Git-admitted and available in the filtered Nix source.
- The same fixture is intentionally rejected by rustfmt and carries a dependency-local denied Clippy lint, while package-scoped format and strict `--no-deps` Clippy checks pass for `lattice`.
- A tracked JSON fixture is parsed from the filtered test closure.
- The maintained full-check command uses only a command-scoped empty signing-key override, and the flake embeds no signing policy.

A hostile control showed `secret-key-files = /definitely/missing-lattice-signing-key`; the same config plus `--option secret-key-files ''` showed an empty effective value and the full flake check passed. Advisory review identified the need for this control; deterministic Nix output remains authoritative.

## Coverage

Coverage remains `798 / 483 / 315` requirements/referenced/missing, with no missing additions or removals, zero dangling markers, and `valid: false`.

- missing-set BLAKE3: `2b7906ee39473996244bcfbf795e9ee984704c8bbfe2db9017309647c4d8373c`
- input BLAKE3: `8225bed1997a289bb62268b7ab6eab66c3963d47d4fdf4f606f5f6983f08b511`
- receipt BLAKE3: `053b1ec799ed452cfbdd95cd7d912ad2be417c4a26f458a65b19d90db5897d94`

Inventory is 315 total, 11 reviewed, 11 exact, and 304 unassessed. Review-policy BLAKE3 is `aaefffa042f72808c2caeba51bb03ade618edd6fd07f86bb291d6cd57df876e0`; inventory BLAKE3 is `2d73e5329ff6b44822c6cd64d14bed827609f723d5d8cd81d5069b96fac25971`.

## Validation

Positive and negative repository-contract tests, historical audit validation, completion validation, direct vendor Clippy rejection, package-scoped format, strict package Clippy, inventory generation/checking, Cairn validation, and the hostile-config full Nix flake check pass.

## Non-claims

The evidence is bounded to the maintained fixture and current repository configuration. It does not prove arbitrary Nix, Git, Cargo, formatter, lint, signing, release, or production behavior.
