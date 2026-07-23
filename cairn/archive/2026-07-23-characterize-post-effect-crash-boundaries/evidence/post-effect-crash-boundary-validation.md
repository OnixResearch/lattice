# Post-effect crash-boundary validation

Date: 2026-07-23

## Result

The scheduler and webhook operator tests now cross a real local effect checkpoint before terminating the owning Lattice process. Both tests inspect a copied crash database before reopening the original. The scheduler profile observes two identified local attempts and one restarted terminal run. The archived webhook baseline observes an incomplete first response, two identified local attempts for one trigger-scoped key, and only the redelivered terminal run plus accepted trigger record.

This establishes bounded local at-least-once risk on the checked Unix/x86_64-Linux host. It does not prove remote commit identity, provider deduplication, external cancellation, arbitrary kill safety, unsupported-host behavior, or exactly-once external execution.

## Baseline

Before implementation, the existing scheduler process matrix and sequential webhook duplicate test passed:

- `nix develop -c cargo test --test test_scheduler operator_e2e_scheduler -- --nocapture`
- `nix develop -c cargo test --test webhooks operator_listener_loads_persisted_trigger_and_accepts_loopback_request -- --exact --nocapture`

## Focused evidence

- `nix develop -c cargo test --test test_post_effect_crash_profile`
- `nix develop -c cargo test --test test_scheduler operator_e2e_scheduler_post_effect_death_repeats_reviewed_attempt_after_restart -- --exact`
- `nix develop -c cargo test --test webhooks operator_listener_post_effect_death_matches_duplicate_risk_profile_on_redelivery -- --exact`
- Pure negative cases reject missing checkpoints, premature terminal records, unreviewed attempt counts, cross-ingress observations, and unsupported HTTP outcomes.
- Real process cases require signal termination, exact attempt counts, copied-store facts, and exact post-restart durable state.

## Repository rails

- `scripts/check.rs`: passed, including operator E2E, package-scoped format, tests, all-target compilation, Clippy, and Cairn spec checks.
- `nix develop -c octet-precommit-check`: 902 accepted findings, 0 new, 0 stale.
- Cairn validation and proposal/design/tasks gates: passed.
- Accepted-spec sync: `lattice.repository_validation.post_effect_crash_boundaries` and all scenarios are present in `cairn/specs/repository-validation/spec.md`.
- Traceability: 905/905 accepted requirements referenced after sync.
- `nix flake check -L --option secret-key-files ''`: all x86_64-Linux checks passed; aarch64-darwin, aarch64-linux, and x86_64-darwin were omitted as incompatible systems.
