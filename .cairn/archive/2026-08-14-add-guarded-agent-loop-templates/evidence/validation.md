# Guarded agent-template validation

Date: 2026-08-14

## Implementation checks

The following checks passed:

```text
CARGO_TARGET_DIR=target/guarded-loop-impl cargo test --test test_guarded_agent_templates
CARGO_TARGET_DIR=target/guarded-loop-impl cargo test --test test_workflow_templates
CARGO_TARGET_DIR=target/guarded-loop-impl cargo test --test test_orchestration_shell
CARGO_TARGET_DIR=target/guarded-loop-impl cargo test --test test_run_inspection
CARGO_TARGET_DIR=target/guarded-loop-impl cargo test --test test_traceability_repository_validation_completion
CARGO_TARGET_DIR=target/guarded-loop-impl cargo test --lib
CARGO_TARGET_DIR=target/guarded-loop-impl cargo check --lib --bin lattice
CARGO_TARGET_DIR=target/guarded-loop-impl cargo clippy -p lattice --lib --bin lattice --no-deps -- -D warnings
CARGO_TARGET_DIR=target/guarded-loop-impl scripts/check.rs
octet-precommit-check
nix shell nixpkgs#nickel --command nickel typecheck policy/workflow-template-bundle.ncl
nix shell nixpkgs#nickel --command nickel typecheck builtins/workflow-templates/cairn-agent-drain/manifest.ncl
nix shell nixpkgs#nickel --command nickel typecheck builtins/workflow-templates/cairn-agent-drain/workflow.ncl
nix shell nixpkgs#nickel --command nickel typecheck builtins/workflow-templates/cairn-agent-drain/orchestration.ncl
nix shell nixpkgs#nickel --command nickel typecheck builtins/workflow-templates/octet-agent-burndown/manifest.ncl
nix shell nixpkgs#nickel --command nickel typecheck builtins/workflow-templates/octet-agent-burndown/workflow.ncl
nix shell nixpkgs#nickel --command nickel typecheck builtins/workflow-templates/octet-agent-burndown/orchestration.ncl
nix flake check -L --option secret-key-files '' --builders ''
```

The guarded-template suite passed 19 tests. The workflow-template suite passed 17 tests. The orchestration-shell suite passed 15 tests. The run-inspection suite passed 7 tests. The focused traceability suite passed 2 tests. The library suite, repository script, and all nine local flake checks passed.

The first unbounded Nix attempt used a remote builder. That builder returned a rust-source store hash mismatch. The final check disabled remote builders and passed locally.

## Bundle identities

The final `cairn-agent-drain` bindings are:

- workflow: `3dc4092a5cea6db8dd2368b86bf55711e9ac9f5a8d1226d1b7723b3ceec8fcff`;
- orchestration: `27e5e3ba57a7d33eb90b906c76532a87797d15df6efb0ff1cb2603a0f2937e1c`;
- prompt: `920a8c43388f3265d70dd877736dfc4ce630ac0031fa45db2a9fb7fbde3f54e4`.

The final `octet-agent-burndown` bindings are:

- workflow: `1dadbc1aa533861862ce29f1baba5eb309961793895bd927d8ef65bf78000885`;
- orchestration: `4036801e8585377dcbce57fe7eea75ed9e506fc66863c294ab83b1979db7b0d0`;
- prompt: `e1099b26a401827632cb55120035e9f215526287d2035dfc29b27fe919ba2b15`.

## Fail-closed coverage

Positive and negative tests cover strict improvement, equality, regression, malformed and overflowing observations, failed prerequisites, bounded Cairn selection, symlink rejection, empty work, and retained operator edits.

Octet tests cover bounded probes, malformed and truncated output, exact policy restoration, recovery conflicts, and concurrent policy mutation. Authority tests reject extra agents, credentials, triggers, changed roots, changed commands, changed targets, self-approval, hidden loops, retries, and expanded budgets.

Persisted evidence contains only typed bounded facts and selected-work fingerprints. Tests reject hostile prose and raw selected names. Preview and cancellation remain inert.

## Octet evidence

The pinned Octet gate passed both scopes:

- `lattice`: 856 accepted, 856 current, zero new, zero stale; baseline `b3:36d7c9cd79a3c05d314212ed40957568497d0aabec4a5327d568d8d760714da5`;
- `lattice-core`: 24 accepted, 24 current, zero new, zero stale; baseline `b3:7e0b9fe7f99811d14db2e1f91e3c3090e336d18c148234071298ec9f4ab3f236`.

All 27 initial project-owned findings were repaired before migration. The reviewed migration paired only the unchanged `src/orchestration/shell.rs` file-length finding. The migration receipt is in `octet-migration-2026-08-14.md`.

## Cairn lifecycle

Lattice policy provenance pins Cairn revision `b6759012db4dab0158936e995b7bad39ac10465b`. Strict validation used that exact revision and the tracked `lattice-default` policy.

Strict validation passed with no issues. Proposal, design, and task gates passed with these receipt hashes:

- proposal: `3d5ea2504acb51ac00018c575abaa3dbe88155e6b39ad78614658456db1f065d`;
- design: `dfa38557b05f268f1cd5fe3ab1a46302958e1a1cc8be8294d664609036dfa1c7`;
- tasks: `889a395b965b010ed592828e7f7b45d1f7e8e58e89c0706509afff0f163cf433`.

All 19 tasks are complete. Sync added six requirements and 15 scenarios to `.cairn/specs/guarded-agent-loop-templates/spec.md`. The accepted spec content hash is `cdedc6dcf536176f3d6f92ea148d43f74ce2a5a67d0c37ce3465f1e5e81a2c5d`. The sync receipt is `4f348ff1cafb8f1d2a529e978aa96be80f3e9ee026870efb947c0ad9e2043111`.

Post-sync traceability has input hash `072f48748fa690818743edc5b11ea286d616e5cdb90dccded035a998496726b5` and receipt `b09c1c04c3b5e35bb58241a909c6fdf68abb0cc4cf0df1b788af99af69519e35`. No `lattice.guarded_agent_templates` identity is missing or dangling.

The full advisory traceability profile still reports existing unrelated debt. Its three dangling identities are:

- `cairn.lifecycle_directory.hidden_default`;
- `lattice.repository_validation.octet_baseline_migration.scenario.dynamic`;
- `lattice.repository_validation.octet_baseline_migration.scenario.resolved`.

## Claim boundary

These checks prove the listed bounded behavior for the checked source, fixtures, policy, and tool revisions. They do not prove model correctness, command safety, provider behavior, exactly-once effects, crash-free execution, external repository truth, or release readiness.
