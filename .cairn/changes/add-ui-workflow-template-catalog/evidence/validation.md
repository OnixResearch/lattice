# Workflow-template catalog validation

Date: 2026-08-14

## Implementation checks

The following checks passed:

```text
CARGO_TARGET_DIR=target/workflow-template-check cargo test --test test_workflow_templates
CARGO_TARGET_DIR=target/workflow-template-check cargo test --lib
CARGO_TARGET_DIR=target/workflow-template-check cargo test --tests --no-fail-fast
CARGO_TARGET_DIR=target/workflow-template-check cargo clippy --package lattice --all-targets --no-deps -- -D warnings
CARGO_TARGET_DIR=target/workflow-template-check scripts/check.rs
nix run nixpkgs#nickel -- typecheck policy/workflow-template-bundle.ncl
nix run nixpkgs#nickel -- typecheck builtins/workflow-templates/shell-preview/manifest.ncl
nix run nixpkgs#nickel -- typecheck builtins/workflow-templates/shell-preview/workflow.ncl
nixfmt --check flake.nix
nix flake check -L --option secret-key-files '' --builders ''
```

The focused workflow-template suite passed 17 tests. The library suite passed 111 tests. The full repository script and all nine flake checks passed.

The built-in workflow identity is `ac10d0357c3fe251f711f81de82735f7eb4b5bd0c9f8a83059137462e260e6c1`.

## Fail-closed evidence

Tests reject malformed or stale identities, traversal, symlinks, non-regular files, undeclared files, and all configured size overflows. They also reject save and activation drift.

A blocked activation creates no orchestration instance, run, or activation receipt. Preview and cancellation do not write the store or dispatch effects.

Initialization tests cover fresh creation, existing targets, and a race after preflight. Existing operator bytes remain unchanged.

## Exact traceability receipt refresh

Implementation changed exact reviewed evidence bytes. Only these reviewed rows were refreshed:

- Two core-credential rows now bind `src/workflow_format.rs` at `999ae734534945abad8c320ef2d50fa70ef7cc59e647ceecaab8988961f4ad02`.
- Three repository-validation rows now bind `README.md` at `bb361f360406f61d43580f82c08f6c93c7cadd617cafc6ed1e1acc8b6905e49b`.
- Two webhook-auth rows now bind `src/store/mod.rs` at `04ad7041e73dbc5d63b59c5b0cefc81430d255aef679443498dc397fdbb1d8cd`.

Each positive receipt test and its fail-closed test passed. The historical flake review fixture and its hashes remain unchanged.

## Octet evidence

`octet-precommit-check` passed both checked baselines:

- `lattice`: 856 accepted, 856 current, zero new, zero stale; baseline `b3:ea0aec4872a3d77db6bdc0cdb39fec49cbc9e4fce2e5fdcb6c3abeef396984bf`.
- `lattice-core`: 24 accepted, 24 current, zero new, zero stale; baseline `b3:7e0b9fe7f99811d14db2e1f91e3c3090e336d18c148234071298ec9f4ab3f236`.

The reviewed migration pairs only semantic owner and line movement. Two assertion-density findings were repaired instead of migrated. The migration receipt records new drift `b3:7803098feba352ecf61a66af48a1402fc9ed85c647cebd720e63b3b5ed7ca5fb` and stale drift `b3:42e4b245ad17172759cf2be8e238be2c49b4c5ad06dc6156663912b22eef6aba`.

## Nix source closure

The first local flake attempt excluded untracked Rust module files. After staging, the next attempt exposed a missing `builtins/workflow-templates/` source-filter rule.

The filter now admits only that built-in asset subtree. The final local flake check compiled the assets through `include_bytes!` and passed all checks.

## Traceability and lifecycle

Cairn strict validation and the proposal, design, and task gates passed. All 15 tasks are complete.

The sync added seven requirements and 15 scenarios to `.cairn/specs/workflow-templates/spec.md`. The post-sync traceability input hash is `62611d53f1fe8e8eec8a92edcfe583ebea5054eb35a6c4d3ec7aac3c779bb86e`.

No `lattice.workflow_templates` identity remains missing or dangling. Existing unrelated traceability debt remains outside this change.

## Claim boundary

The evidence proves bounded loading, validation, preview, initialization, saving, and activation behavior for the checked inputs. It does not prove bundle behavior, command safety, external effect uniqueness, authority, or release readiness by itself.
