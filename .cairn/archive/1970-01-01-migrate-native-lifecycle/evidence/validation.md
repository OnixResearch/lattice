# Native lifecycle migration evidence

## Source and plan

- Cairn policy revision: `695124d459574ba7aeba6097310d237f393c243c`
- migration plan hash: `a15c15cde3834704ec43e413a02a22494aa34c1b57d29e33895769a4719099f0`
- source manifest hash: `08907b661e89f55fc7ce0041a8a13f06b8ac3e157c1e4ba61eb8043e0b249a51`
- destination manifest hash: `ab74d5d81a0a571898b7ee0c18dd518adc352c8e8d3ec9610808634a8d15c3c4`
- preview receipt file BLAKE3: `b946869ec34710f101e6b04f22fa5aa86586861a92504698d87519428f7ce816`
- execution receipt file BLAKE3: `cf8daac8b6615c645cc2e00c8536552004cf8f136c814cfae1a78870d076ef01`

The reviewed plan had no conflicts. Execution copied the complete legacy tree before active paths changed and before `cairn/` was removed.

## Policy

The project policy imports an exact upstream snapshot and overrides only Lattice traceability roots and the project name.

- generated policy BLAKE3: `a76a5d73e44186aeaceb4a5470b05d4933ebe17f3360f10940d43f602aa59677`
- `cairn policy export --check`: passed in the pinned Cairn development shell
- repeated Nickel exports: byte-identical
- unsafe traceability-root fixture: rejected
- legacy-only lifecycle fixture: rejected with `cairn.lifecycle_directory.migration_required`

## Validation

- `scripts/check.rs`: passed; one orchestration child fixture failed once and passed its bounded retry
- focused lifecycle, traceability, identity, and marker tests: passed
- `octet-precommit-check`: passed with the reviewed baseline unchanged
- strict Cairn validation: passed
- proposal, design, and tasks gates: passed
- `nix flake check -L --option secret-key-files ''`: passed all host checks

Nix omitted `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` as incompatible host systems.

## Claim boundary

This evidence proves the selected migration and validation observations only. It does not prove requirement truth, product correctness, external dependency correctness, or release readiness.
