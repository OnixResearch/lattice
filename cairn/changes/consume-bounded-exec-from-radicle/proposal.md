## Why

Lattice already consumes Bounded Exec at exact revision `29dac88ecded94457572db3fdfaaaab95fa91525`, but Cargo and Nix acquire it through GitHub SSH URLs. This keeps an Onix-owned runtime dependency coupled to GitHub availability even after the same Git object is published through Radicle.

The safest pilot changes only source transport and repository identity. The Bounded Exec revision, crate API, process behavior, Lattice policy, and runtime tests remain fixed so failures can be attributed to source acquisition rather than a dependency upgrade.

## What Changes

- Replace the Bounded Exec Cargo and Nix GitHub source URLs with the accepted Radicle-backed HTTPS Git endpoint while retaining the exact revision.
- Regenerate Cargo and Nix lockfiles with their owning tools and align Crane's vendored Git source identity with the new canonical source string.
- Add deterministic source-identity checks that reject RID, revision, lock, or content mismatch before accepting a build.
- Prove acquisition through each admitted seed path and prove failure does not silently fall back to GitHub.
- Preserve all current bounded-process behavior and focused positive/negative runtime tests.

## Impact

- **Files**: `Cargo.toml`, generated `Cargo.lock`, `flake.nix`, generated `flake.lock`, source-identity checks/fixtures, README reference metadata, and Cairn evidence.
- **Cross-repo inputs**: accepted Bounded Exec publication evidence and OnixOS endpoint/availability evidence.
- **Testing**: source consistency positives and negatives, focused bounded-process tests, repository checks, Octet, Cairn gates, and bounded Nix validation.
