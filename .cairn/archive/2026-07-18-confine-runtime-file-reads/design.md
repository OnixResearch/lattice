## Context

The file-effect shell now uses `cap-std`, but three runtime readers retain check-then-open behavior. Image input performs only lexical admission before metadata and read calls. Prompt hydration canonicalizes then opens. Agent handoff canonicalizes and obtains metadata before a separate read. All can resolve a different target after a concurrent replacement.

## Goals

- Make every authority-bounded runtime read descriptor-relative.
- Share pure path admission and bounded byte acquisition.
- Keep caller-specific decoding and diagnostics outside the generic capability module.
- Test the race boundary deterministically by replacing an intermediate component after opening the root.

## Non-goals

- Converting unrestricted CLI import/export paths that are direct operator arguments.
- Sandboxing untrusted workflow definitions or configured root paths.
- Preventing access through hard links or mounts intentionally exposed beneath a granted root.

## Decisions

### Shared functional core

A capability module owns a pure rooted-path value. It selects the most-specific configured root for absolute or root-prefixed targets and derives safe non-empty relative paths for root-relative targets. Parent, absolute-relative, prefix, and empty components fail admission.

### Shared imperative shell

The module opens a configured root with explicit ambient authority, then opens only admitted relative paths through the resulting `cap_std::fs::Dir`. Bounded reads consume at most `max_bytes + 1` bytes from the opened file handle, allowing deterministic overflow detection without a second pathname lookup.

### Caller-owned semantics

- `LocalFs` keeps operation-specific write/append options.
- Image input requires an explicit allowed root, validates the opened handle's metadata length, performs a bounded read on that same handle, and retains media sniffing.
- Prompt hydration opens the workflow directory as a capability, reads each validated relative Markdown path through that handle, and retains UTF-8/non-empty checks.
- Agent handoff opens command `cwd`, admits relative paths or absolute paths beneath an absolute `cwd`, and obtains metadata plus bounded bytes from the same file handle.

### Root acquisition boundary

Configured roots, workflow directories, and command working directories are trusted authority grants. Ambient access is used only to acquire those root handles. This change protects target and intermediate resolution after root acquisition; it does not claim to sandbox malicious configuration or a parent that can replace the granted root before acquisition.

## Alternatives

- Per-reader `canonicalize` repairs remain vulnerable to external path replacement.
- A Linux-only direct `openat2` layer duplicates the already-selected portable `cap-std` mechanism.
- Process-local locks cannot constrain agent or external filesystem actors.

## Validation

Focused image, prompt, agent, and file tests run before and after. New tests cover missing image roots, bounded reads, static symlink escapes, and post-root-open intermediate replacement. Full repository, Clippy, Octet, Cairn, and check-scoped Nix flake rails must pass.
