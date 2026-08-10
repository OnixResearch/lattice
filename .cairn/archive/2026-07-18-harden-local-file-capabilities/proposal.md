# Change: Harden local file capabilities

## Why

`LocalFs` currently canonicalizes a path for containment and then reopens the original ambient path. An attacker able to rename a checked directory or replace it with a symlink between those operations can redirect a read, write, or append outside the configured root. Static containment checks therefore do not establish containment at the side-effect boundary.

## What Changes

- Require production local-file effects to name at least one allowed root.
- Select a root and relative path in a deterministic, I/O-free core.
- Open the selected root once as a directory capability and perform reads, writes, and appends relative to that descriptor.
- Use the portable `cap-std` implementation, which uses `openat2` where available and descriptor-by-descriptor resolution elsewhere.
- Add positive nested-I/O tests and negative tests for missing roots, traversal, escaping symlinks, and parent-path replacement.

## Impact

- Affected spec: `workflow-execution`
- Affected code: `src/file_node.rs`, focused file-node tests, Cargo dependency metadata
- Compatibility: `MemoryFileSystem` keeps its existing lexical policy; production `LocalFs` now fails closed when no root capability is configured
