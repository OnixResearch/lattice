## Context

The host sets `secret-key-files = /run/secrets/vars/nix-signing-key/key` globally. A missing runtime secret therefore prevents the daemon from finalizing any new derivation, including checks that do not need signed outputs. Repository flakes cannot safely override this host policy internally before builds begin. A caller-scoped `--option secret-key-files ''` override is already an established Onix check/runbook mechanism and leaves ordinary builds unchanged.

The flake source contains path dependencies under `vendor-cargo-deps/`. Crane's generic Cargo source filter retains those files for builds, while `cargo fmt` without a package selector traverses dependency sources. Unanchored `.gitignore` entries also matched nested Rust paths named `result*` and `coverage/`, causing the working tree to contain required files that were absent from Git/Nix snapshots.

## Decisions

### Decision: Keep signing policy outside the flake

**Choice:** Document the unsigned check invocation at the command boundary instead of adding flake `nixConfig` that weakens signing for every consumer.

**Rationale:** Validation outputs do not need machine signatures. Normal package, cache, and release invocations continue to inherit the host's signing policy, while checks remain runnable on machines whose signing secret is intentionally absent.

### Decision: Preserve complete inputs but narrow tool ownership

**Choice:** Keep vendored path dependencies and tracked integration fixtures in the build source. Pass `--package n8nclone` to Crane's format check and `--package n8nclone --no-deps` to its Clippy check.

**Rationale:** Builds and integration targets require the exact snapshots and non-Rust fixtures, but formatting or linting imported source would mutate provenance or apply consumer policy to upstream code. The package selectors match the existing repository-local formatting and Clippy rails.

### Decision: Anchor output ignores

**Choice:** Treat build, coverage, documentation, data, Nix result, and direnv paths as repository-root outputs.

**Rationale:** Nested dependency files may legitimately use those names. Root anchoring prevents valid source from disappearing from Git and Nix source snapshots while retaining the intended local output exclusions.

## Risks / Trade-offs

- The documented check override produces unsigned local check outputs; it does not prove host signing-key availability or release-cache signing.
- Vendored snapshots remain immutable provenance inputs except through the repository sync process; tracking omitted files repairs snapshot completeness rather than accepting arbitrary vendor edits.
- A future workspace expansion must deliberately update the format and Clippy package selectors or provide a bounded package list.
