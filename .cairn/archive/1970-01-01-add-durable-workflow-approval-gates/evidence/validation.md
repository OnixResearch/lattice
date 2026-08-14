# Durable workflow approval validation

## Product and compatibility checks

The implementation commit is `d099122d26156e0d54078e78625dcb7e9db208bd`.

The focused approval suite passed with 14 tests and one child-only fixture ignored by the parent test runner. The suite covers typed configuration, authority denials, expiry, rejection, decision and resume separation, opaque credentials, multiple gates, process restart, process kill after atomic commit, CLI behavior, TUI behavior, and workflow-format compatibility.

```text
CARGO_TARGET_DIR=target/workflow-approval \
  nix develop -c cargo test --test test_workflow_approval -- --nocapture
```

`CARGO_TARGET_DIR=target/workflow-approval scripts/check.rs` passed. This gate includes formatting, all repository tests, operator E2E processes, focused source guards, and Clippy for all Lattice targets with `--no-deps -- -D warnings`.

The existing orchestration identity compatibility tests passed. The frozen pre-approval workflow revision and Nickel projection hashes remain unchanged.

The reviewed pinned Choregraph downstream command passed two tests:

```text
CARGO_TARGET_DIR=/home/brittonr/git/OnixResearch/choregraph/target/lattice-approval-pin \
  nix develop -c cargo test -p choregraph-seaglass --test lattice-suspension -- --nocapture
```

These tests cover suspended wait, matching resume, mismatched denial, and deterministic replay for revision `0b2000edaf86d5a7a8601246afbc0d526f4a2a07`.

## Octet

`nix develop -c octet-precommit-check` passed with warning-only status. The top-level baseline has 858 accepted and 858 current identities, with zero new and zero stale identities. Its baseline hash is `b3:de11a299574a712aaa04b45eb266abf14480a20e389ea6eeb9f37283fac3cc28`.

The independent `lattice-core` baseline has 24 accepted and 24 current identities, with zero new and zero stale identities. Its baseline hash remains `b3:7e0b9fe7f99811d14db2e1f91e3c3090e336d18c148234071298ec9f4ab3f236`.

See `octet-migration-2026-08-14.md` for the exact reviewed migration and repaired finding.

## Cairn

Strict repository validation passed. Proposal, design, and task gates passed for `add-durable-workflow-approval-gates`.

The dry-run sync plan was unblocked. It planned seven added requirements in `.cairn/specs/workflow-execution/spec.md` with expected post-sync hash `916a0ad647e6cf32309b93fbf10baec25dce6cc3685fa35461f0d50c59a99e07`.

The pre-sync global traceability command reported the new approval markers as dangling because change-local requirements were not accepted until sync. After sync, strict validation and the task gate passed again. The global traceability receipt `d35a482c95f518068d92bc83c9764734c2131f34e98d745226cdd2adc60a3bef` contains no dangling or missing `lattice.workflow_approval` identity.

The global traceability command still returns failure for three unrelated dangling identities and the repository's existing missing-requirement inventory. This change does not promote or hide that existing debt.

## Nix

The default remote-builder run evaluated all outputs and started all nine checks. It stopped because the remote builder imported a corrupted `rust-src` store path with a hash mismatch. This was an infrastructure fault, not a source diagnostic.

The same gate was rerun locally with `--builders ''`. Its first Nextest attempt had one transient bounded-process spawn failure. A targeted rebuild then passed all 898 Nextest tests, with one declared skip. The package derivation was rerun separately after a later transient child-workflow spawn failure, and it passed all release tests.

The final local `nix flake check -L --option secret-key-files '' --builders ''` reused the verified outputs and reported `all checks passed!` for all nine checks. It omitted incompatible non-host systems as declared by Nix.

## Non-claims

These checks establish the recorded Lattice behavior for the selected source, toolchains, stores, and fixtures. They do not prove external human identity, reviewer independence, credential freshness, remote service behavior, Choregraph properties outside the pinned profile, or whole-system correctness.
