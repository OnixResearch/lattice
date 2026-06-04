# n8nclone

## Development

```bash
nix develop          # enter devshell
scripts/check.rs     # run fmt, tests, and clippy with repo-local target artifacts
cargo check          # quick compile check
nix flake check      # full CI (build + test + clippy + fmt)
```

## Examples

Complex templates and runnable Nickel workflow examples live under `examples/`, including daily digest, CSV cleanup, and log triage workflows.

```bash
n8nclone --config ./demo/config.ncl --init
n8nclone --config ./demo/config.ncl import examples/workflows/daily-digest.ncl
n8nclone --config ./demo/config.ncl run daily-digest --output json
```

## Verification

```bash
verus verus/example_spec.rs
```
