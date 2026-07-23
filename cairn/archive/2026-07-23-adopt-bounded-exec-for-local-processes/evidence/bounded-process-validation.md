# Bounded Local-Process Validation

Date: 2026-07-23

## Source identity

- Canonical source: `git@github.com:OnixResearch/bounded-exec.git`
- Reviewed immutable revision: `29dac88ecded94457572db3fdfaaaab95fa91525`
- License at that revision: `AGPL-3.0-or-later`
- `git ls-remote ... refs/heads/main` returned the exact reviewed revision.
- Cargo and Nix use the same SSH repository and revision. Crane vendors the locked dependency from the exact Nix flake input store source; no sibling path is admitted.
- Positive and negative repository tests reject path substitution, lock drift, and malformed or absent lock identities.

## Implementation evidence

- `src/process/core.rs` is the pure admission core for explicit executable, argv, environment mode and overrides, current directory, stdin, independent stream limits, deadline, polling, teardown, accepted exits, truncation policy, and termination scope.
- `src/process/shell.rs` is the thin imperative shell for executable/current-directory resolution, fixed diagnostic translation, cancellation, and `bounded-exec` invocation.
- Shell nodes use a Tokio `spawn_blocking` bridge. Command agents and command credential providers use the same `ProductionProcessRunner` boundary.
- Repository source scan found no remaining `Command::new`, `std::process::Command`, `tokio::process::Command`, or `wait_with_output` mechanics under `src/`.

## Positive and negative checks

The pre-change focused shell, command-agent, credential-provider, headless, and repository-validation suites passed.

The final checks passed:

- `scripts/check.rs`: repository formatting, complete package tests, strict project-only Clippy, and accepted Cairn spec checks.
- `cargo test --test test_local_process`: positive bounded execution plus malformed-policy, missing path, stdin overflow, stdout/stderr flood, mixed output, timeout, cancellation, post-timeout side-effect, descendant-held-pipe, command-agent, credential-provider, and diagnostic-redaction cases.
- Repeated credential-log capture: timeout and truncation cases ran three times each; captured logs contained neither `late-secret` nor `must-not-be-returned`.
- Octet migration self-test and dry run: `old=1159 candidate=1151 matched=1151 stale=8 new=0`.
- Octet zero-delta wrapper: 902 accepted/current error-level findings, zero new, zero stale; baseline BLAKE3 `692c91daa53ec8d91ce2fc94ec09f93d2616bee8d52fd8b9db2f93fe4660677f`.
- `nix flake check -L --option secret-key-files ''`: all configured `x86_64-linux` checks passed. Incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems were omitted.
- Cairn proposal, design, and task gates passed before implementation; repository validation passed before synchronization.

The eight resolved Octet findings were the removed direct `Command` imports, direct runner assertion debt, old seven-parameter shell wrapper, old shell boolean naming, ignored stdin-write result, and explicit must-use drop. No new Octet debt was accepted.

## Non-claims

This evidence is bounded to the declared local executable, environment, directory, input, limits, process-group behavior, and observed tests. It does not prove sandboxing, executable authorization, hermeticity, syscall restriction, arbitrary host-process cleanup, Windows descendant teardown, child semantic correctness, remote cancellation, external provider truth, or whole-system correctness.
