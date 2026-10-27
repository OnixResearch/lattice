## Why

The reviewed UCAN snapshot pins BLAKE3 `1.8.4` in peer-authentication crates. Current Onix consumers require exact `1.8.5`, so Cargo cannot compose Basalt with those consumers.

## What Changes

- Import the reviewed UCAN BLAKE3 alignment revision with the repo-owned importer.
- Regenerate Basalt's lock through Cargo.
- Admit Molten's reviewed `iddqd` `0.4.5` source cohort without changing Basalt's locked `0.4.6` build.
- Verify the snapshot manifest, branch-authority API, no-default library, CLI, release rail, and downstream composition.
- Preserve Basalt and UCAN authority boundaries.

## Impact

This changes one dependency cohort. It does not change policy meaning, token semantics, hash framing, key authority, replay, revocation, or release claims.
