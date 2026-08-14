# Manifest-driven agent-evaluation validation

Date: 2026-08-14

## Implementation checks

The following checks passed:

```text
rustfmt --edition 2024 --check <changed Rust files>
CARGO_TARGET_DIR=target/agent-eval cargo test -p lattice --test test_agent_evaluation_core --test test_agent_evaluation_shell
CARGO_TARGET_DIR=target/agent-eval cargo clippy -p lattice --lib --bin lattice --no-deps -- -D warnings
CARGO_TARGET_DIR=target cargo clippy --package lattice --all-targets --no-deps -- -D warnings
CARGO_TARGET_DIR=target/agent-eval-check scripts/check.rs
nix shell nixpkgs#nickel --command nickel typecheck <six evaluation policy, catalog, request, manifest, and workflow files>
octet-precommit-check
nix flake check -L --option secret-key-files '' --builders ''
```

The focused core suite passed 12 tests. The focused shell suite passed nine tests. The full repository script passed its format, boundary, focused, full test, Clippy, and Cairn-spec checks. All six Nickel files typechecked. All 11 local flake checks passed.

The repository-wide Cargo format check still reports only the intentional nonconforming fixture at `tests/fixtures/repository-validation/vendor-dependency/src/coverage/result.rs`. Every changed Rust file passes `rustfmt --check`.

The first Nix attempt used a remote builder and was stopped after 34 minutes of remote source-transfer and unpack latency. The final bounded check disabled remote builders and passed locally.

## Product evidence

Positive tests cover all four case classes, strict catalog loading, deterministic planning, exact resume, hard-gate reduction, advisory judgment, fake-live deployment and cleanup, bounded artifacts, metadata-only receipts, durable redb restart, and the production command-driver protocol.

Negative tests cover malformed and uppercase identities, duplicate or stale catalog data, asset drift and symlinks, unavailable services, insufficient budgets, production targets, missing live authority, subject drift, hard-gate failure, judge override, secret-bearing artifacts, path escape, oversized artifacts, private Animus facts, driver identity drift, execution uncertainty, cleanup failure, and duplicate durable start.

The production driver check proves that durable `Started` state exists before `agent_execute`. A response with a changed driver, trial, case, profile, environment, policy, or operation identity fails before durable trial start. Drivers use absolute executable paths, a cleared environment, bounded input, output, and time, and metadata-only JSON.

Live execution remains preview-first without `--confirm-live`. Resume derives prior claims only from validated durable trial records and matching receipts.

## Exact traceability receipt refresh

Implementation changed exact reviewed evidence bytes. Only these historical reviewed rows were refreshed:

- Three repository-validation rows now bind `README.md` at `d95e6c0847f82d7360166ed9be3586bd0d21d6fae9e4b0b6174419cdf873e612`.
- Two webhook-auth rows now bind `src/store/mod.rs` at `2fa34e36ca58faca0ea1970d72953b55e671daabf4b2d7e61d07b498e95da3c1`.

Each positive receipt test and its fail-closed drift test passed.

## Octet evidence

The pinned Octet gate passed both scopes:

- `lattice`: 856 accepted, 856 current, zero new, zero stale; baseline `b3:3212f34e0b57ed9e3e49aff1f5f7cf6267b627b9e4b9498a0da09934eb04a642`;
- `lattice-core`: 24 accepted, 24 current, zero new, zero stale; baseline `b3:7e0b9fe7f99811d14db2e1f91e3c3090e336d18c148234071298ec9f4ab3f236`.

All 114 initial project-owned findings were repaired before migration. The reviewed migration paired only the unchanged `src/store/mod.rs` file-length finding. See `octet-migration-2026-08-14.md`.

## Cairn lifecycle

Lattice policy provenance pins Cairn revision `b6759012db4dab0158936e995b7bad39ac10465b`. Strict validation used that exact revision and the tracked `lattice-default` policy.

Strict validation passed with no issues. Proposal, design, and task gates passed with these receipt hashes:

- proposal: `46ad5dc0036a0677bc536d9e668a8806bf4f41da1f39556c20ffcdd94341374b`;
- design: `420d05a654747980009b59139443eabf62e74d052ddea032ef46b176871ef79b`;
- tasks: `a0d69829506ee146d13dd830a475055e096cc6facc14027dd6c9803a9e885dde`.

All 39 tasks are complete. Sync added eight requirements and 23 scenarios to `.cairn/specs/agent-evaluation/spec.md`. The accepted spec content hash is `6580c648885d6065363d308bf8944ebb8a0f75efa5fd7a39e7ff584c338e1d13`. The sync receipt is `a56b491c01cb6b02121a76995e6749cac6ee16cd8cb8447568611fb516309865`.

Post-sync traceability has input hash `5a1868a939008cd559b609d25d563a73935e89fc1958ff584bca21f4cddd993f` and receipt `e5a4dbf9baa01bb1284a6333eaf7008b3673ab8f42f8f3673b20a34976c51740`. No `lattice.agent_evaluation` identity is missing or dangling.

The full advisory traceability profile still reports existing unrelated debt. Its three dangling identities are:

- `cairn.lifecycle_directory.hidden_default`;
- `lattice.repository_validation.octet_baseline_migration.scenario.dynamic`;
- `lattice.repository_validation.octet_baseline_migration.scenario.resolved`.

## Claim boundary

These checks prove the listed bounded behavior for the checked source, fixtures, policy, and tool revisions. They do not prove agent correctness, verifier soundness, judge correctness, model quality, benchmark superiority, provider behavior, remote effect uniqueness, deployment correctness, production readiness, release eligibility, or release readiness.
