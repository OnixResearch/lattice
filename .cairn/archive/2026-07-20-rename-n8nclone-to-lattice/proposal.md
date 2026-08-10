## Why

The `n8nclone` name describes an origin rather than the product's graph-oriented runtime and now conflicts with its independent execution, policy, evidence, and agent-workflow identity. The project needs one durable public name before its next release and before external consumers depend on its executable, crate, or Nix output names.

## What Changes

- Make **Lattice** the canonical product name and `lattice` the Rust package, crate, executable, and Nix package/check identity.
- Update current source, tests, examples, operator documentation, generated headers, and user-visible runtime strings to the Lattice identity.
- Introduce Lattice-native state/config names while admitting existing `N8NCLONE_DATA_DIR`, `~/.n8nclone`, and `n8nclone.redb` state through explicit compatibility selection.
- Preserve the versioned webhook signature domain, existing Cairn requirement identifiers, and historical archives because they are durable protocol/evidence identities rather than mutable branding.
- Rename the local independent checkout after the lifecycle package is archived; remote repository mutation and pushing remain separate operator-authorized actions.

## Impact

- **Files**: Cargo and Nix metadata, current Rust sources/tests, examples, README, accepted Cairn specs, migration evidence, and the parent workspace ignore entry.
- **Compatibility**: Existing state remains discoverable; explicit old environment and path inputs remain accepted; webhook signatures and historical evidence remain stable.
- **Testing**: Cairn gates/validation, positive and negative compatibility tests, repository checks, strict Clippy, and bounded `nix flake check`.
