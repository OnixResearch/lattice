# n8nclone

## Development

```bash
nix develop          # enter devshell
scripts/check.rs     # run fmt, tests, and clippy with repo-local target artifacts
cargo check          # quick compile check
nix flake check      # full CI (build + test + clippy + fmt)
```

## Verification

```bash
verus verus/example_spec.rs
```
