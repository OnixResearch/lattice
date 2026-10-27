# Vendored UCAN consumer-cohort verification

Verified on 2026-08-27 in the dedicated Basalt worktree.

## Reviewed source

Basalt imports UCAN revision `6f888f6c91a4ea26f0bd52b6486e6643c8f6d271` with the repository-owned importer.

The 693-file snapshot and direct source-parity checks passed. The snapshot manifest BLAKE3 is `a30addca944cb27578f2c9e8fbe8c133a6839ff9aabfa6f0151e436ff920a7be`.

The source owner aligned these exact cohorts:

- BLAKE3 `1.8.5` for peer authentication and Nostr identity;
- Zeroize `1.9.0` for the shell, peer-authentication core, and Molten's Iroh graph;
- implementation-identity inputs for both reviewed changes.

Basalt admits `iddqd 0.4.5` as its minimum compatible API. Its own lock continues to select `0.4.6`.

## Snapshot and focused checks

Five importer unit tests passed. They include positive inventory checks and negative revision, mode, manifest, and license cases.

Four dependency-cohort tests passed. They cover exact BLAKE3 and Zeroize pins, one lock entry for each package, drifted pins, and a duplicate lock entry.

The focused Nostr identity and deterministic receipt checks passed after all identity artifacts were refreshed.

## Release checks

The direct release verifier reports `passed: true`.

Its no-default nextest rail passed 234 tests. Its CLI rail passed 247 tests, and two doctests passed.

Strict Clippy passed for all supported targets in both feature cohorts. Formatting, private package contents, library documentation, and the Clankers fixture passed.

The full Nix flake check passed all 15 supported-system checks. It verified the final UCAN revision and all generated Nickel artifacts.

## Molten consumer check

The final consumer check used Molten `origin/molten` revision `d50c91f2a825f4811404ba9d590626b636472ce5` in a temporary detached worktree.

The temporary lock reconciled `serde_json` from `1.0.150` to UCAN's exact `1.0.149`. No source file changed in the Molten repository.

Cargo then checked the Molten library with this Basalt worktree as a source patch. It selected BLAKE3 `1.8.5`, Zeroize `1.9.0`, and `iddqd 0.4.5`.

The check completed successfully. The temporary worktree was removed after the result.

## Claim boundary

This evidence proves tested snapshot parity and dependency composition only. It does not prove cryptographic correctness, memory erasure, universal compatibility, or Molten release eligibility.
