# Lattice 0.1.0

Lattice is a graph-native workflow and agent runtime with durable local
execution and explicit effect authority. This is a pre-1.0 release candidate.

## Install

Build the tagged source with the pinned development shell:

```sh
git clone https://github.com/OnixResearch/lattice.git
cd lattice
git checkout v0.1.0
nix build .#lattice
./result/bin/lattice --version
```

The release archive contains the `lattice` binary, documentation, and a
`SHA256SUMS` file. Nix users should prefer building `.#lattice` from the tagged
source.

## What is included

- Typed Nickel workflow import/export, validation, and graph execution.
- A TUI and headless CLI with run inspection and recovery commands.
- Fail-closed shell, HTTP, file, and agent effects with exact operator grants.
- Durable runs, schedules, webhook claims, approvals, and orchestration state.
- A versioned workflow contract and local Unix-socket exchange service.
- A public build closure with no SSH or authenticated source requirement for the
  default runtime.

## Verification

The release workflow runs the maintained checks on the tagged commit:

```sh
scripts/check.rs
octet-precommit-check
nix flake check -L --option secret-key-files ''
nix build .#lattice
```

The Octet baseline was migrated to the current workspace/profile cohort with a
delta review against the pre-change HEAD. The traceability evidence fixtures
were refreshed for the current reviewed sources. `nix flake check` passes
1385 tests with 5 skipped.

## Known limitations

- The workflow exchange service is a local Linux Unix-socket service. It has no
  TLS, remote authentication, HTTP listener, or multi-tenant isolation.
- Workflows run with host authority. Lattice is not a sandbox for untrusted
  workflows.
- Three TUI authoring PTY regressions are ignored by the release gate and remain
  tracked by `complete-tui-nickel-authoring-parity`. This release does not claim
  TUI authoring-parity completion.
- The optional Tile compose integration requires authorized Tile source access
  and is not part of the default build or release artifact.
- No hosted service, support SLA, or production-readiness certification is
  included.

## Non-claims

The release proves that the tagged source built and the recorded checks passed
for that revision. It does not prove arbitrary workflow correctness, host
confinement, external effect success, credential freshness, multi-tenant
isolation, or production fitness.
