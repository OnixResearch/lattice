## Context

The current production shell separates canonical containment validation from ambient `std::fs` access. The checked pathname can therefore differ from the pathname opened after a concurrent rename or symlink swap. The repository forbids unsafe code and supports more than Linux, so a direct libc implementation is not an acceptable default.

## Goals

- Bind each local file operation to an already-open allowed-root directory.
- Keep root selection and relative-path admission pure and deterministic.
- Preserve read, write, append, overwrite, and typed error behavior where it remains security-compatible.
- Fail closed before ambient file access when no root or no safe relative path is available.

## Non-goals

- Sandboxing arbitrary Rust code or other filesystem users.
- Preventing access to a file intentionally exposed through an allowed-root hard link or mount.
- Creating missing parent directories.
- Extending this change to image, prompt, or agent handoff file readers.

## Approach registry

### Capability directory

Use `cap_std::fs::Dir::open_ambient_dir` only for the selected configured root, then call `Dir::open` or `Dir::open_with` with the admitted relative path. `cap-std` confines `..`, absolute paths, and escaping symlinks; on Linux it uses `openat2` when available and otherwise resolves components through descriptors.

State: selected.

### Direct `rustix`/`openat2`

A direct `openat2` call can provide the Linux invariant but requires a separately audited fallback for macOS and older kernels. That duplicates mature path-resolution logic and expands platform-specific policy.

State: rejected for portability and audit surface.

### Canonicalization plus process locking

A process-local lock cannot constrain external processes, and canonicalization followed by ambient open retains the original race.

State: falsified.

## Decisions

### Deterministic pure admission

A pure core selects the matching configured root with the greatest component depth and derives a non-empty relative path containing only normal path components. Equal-depth roots retain configuration order. Absolute, parent, prefix, and empty targets fail admission.

### Thin capability shell

The imperative shell opens exactly the selected root using explicit ambient authority. Every target operation then uses only the root capability and relative path. It never canonicalizes and later reopens the original operation path.

### Fail closed without roots

An empty `allowed_roots` list is rejected by `LocalFs`. Production effects must carry explicit authority; unrestricted ambient access is not a safe fallback. The in-memory test implementation remains unchanged because it does not touch host files.

## Risks / Trade-offs

- Adding `cap-std` increases the dependency closure but avoids repository-owned unsafe and platform path-resolution code.
- Capability errors caused by escapes and ordinary OS permission failures may share `PermissionDenied`; both remain fail-closed.
- Opening a configured root still uses ambient authority. The security boundary begins at the resulting directory handle, so configuration controls which roots are granted.

## Validation

- Existing focused file tests must remain green.
- New tests must prove nested read/write/append success and reject no-root, traversal, static symlink escape, and descriptor-root/path replacement cases without modifying outside content.
- Repository checks, Clippy, Octet, Cairn validation, and the full check-scoped Nix flake rail must pass.
