# Public release audit

Observed on 2026-11-11 from the Lattice repository root.

## Scope

This audit records local release-preparation observations. It does not record a
tag, a published release, a deployment, a hosted service, or production
readiness. Unrun or unauthorized actions remain pending.

## Public acquisition

| Input | Revision | Route | Observation |
|---|---|---|---|
| Valence | `15e72b5a330b08139ac7c809ac31e7e87f825b22` | `https://github.com/OnixResearch/valence.git` | Exact revision fetched over anonymous HTTPS. |
| Artifact | `5f3962d7471d1547e044f57daeda72f56621fd52` | `https://github.com/OnixResearch/onix-artifact.git` | Exact revision fetched over anonymous HTTPS and selected by root patch. |
| Trellis | `d65974b14978e1a44b0f5067beb9ef6d83317dc8` | `https://github.com/OnixResearch/trellis.git` | Exact revision fetched over anonymous HTTPS and selected by root patch. |
| Cairn | `47ea1b867c44b026c030b535f9a1041950c482a0` | `https://github.com/OnixResearch/cairn.git` | Flake input and nested Artifact/Mantle inputs use HTTPS overrides. |
| Octet | `bac0f3220ada38110c03498ccf2beae5ee83e642` | GitHub HTTPS | Flake input; nested Cairn and Valence inputs use HTTPS overrides. |
| Tile | `5e703953840bff72a91fec3e928682e9efb607ab` | SSH, authorized only | Excluded from the default workspace, public checks, and release artifact; kept in `compose/` for authorized builds. |

Checks performed:

- `Cargo.lock`: zero `ssh://` entries.
- `flake.lock`: zero `ssh://` entries.
- `cargo metadata --locked --format-version 1 --no-deps`: passed.
- `cairn policy export --check`: `policy fresh`.

## Lifecycle

- `cairn validate --root .`: passed.
- `cairn gate proposal prepare-public-release`: PASS.
- `cairn gate design prepare-public-release`: PASS.
- `cairn gate tasks prepare-public-release`: PASS.

## Build and smoke

Observed:

- `cargo metadata --locked --format-version 1 --no-deps`: passed.
- `cargo check --package lattice --all-targets --locked`: passed.
- `cargo build --package lattice --locked`: passed.
- `cargo test --manifest-path compose/Cargo.toml --locked`: 3 passed with
  authorized Tile source access.
- `lattice --version`: `lattice 0.1.0`.
- `lattice --help`: passed.
- Effect-free smoke: `import` and `export` of the daily-digest example passed;
  `run` without authority exited 1 with a failed run and skipped dependents.
- `nix eval --raw .#packages.x86_64-linux.lattice.name`: `lattice-0.1.0`.
- `nix build .#lattice --no-link -L --option builders ''`: passed. The Nix-built
  binary repeated the version, help, import, export, and fail-closed run results.
- `octet-precommit-check`: passed after baseline migration. Root baseline is
  1453 findings and core baseline is 25 findings. The delta against the
  pre-change HEAD was 1471 findings at HEAD versus 1453 in this tree: 21 findings
  removed, 2 count/message drifts, and no new semantic finding categories.
- `nix flake check -L --option builders '' --option secret-key-files ''`: passed.
  Nextest reported 1385 passed, 5 skipped, 0 failed.
- `scripts/check.rs` (run as `cargo -q -Zscript scripts/check.rs` in the dev
  shell): passed.

## CI publication attempt

Tag `v0.1.0` was pushed to GitHub and the release workflow was run. It did not
publish a release:

- The first run failed before the harness because the release workflow did not
  rewrite GitHub SSH fetches to HTTPS. The workflow now has the same rewrite
  step as the check workflow.
- The next runs failed fetching Bounded Exec from
  `https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git` (HTTP 530).
  The Bounded Exec Radicle seed also returned HTTP 500 for
  `z2LSXA4oFgKeygzLog1K1bqonbjzy`.
- A GitHub mirror exists and can fetch the exact revision, but
  `r[lattice.repository_validation.radicle_bounded_exec_availability.scenario.no-fallback]`
  explicitly forbids a GitHub Bounded Exec fallback. The release did not bypass
  that policy.

The tag and workflow remain in place. Re-running the workflow after the Onix
host or the Bounded Exec Radicle seed provides the exact object is the next
release action. No GitHub release assets exist yet.

Three build corrections were required and are part of this change:

- `cargoArtifacts` now uses the real source instead of crane's dummy source,
  because the external `nickel-lang` crate consumes the path-patched
  `nickel-lang-core` module surface.
- The source filter includes top-level `fixtures/`, which the release-readiness
  integration test includes at compile time.
- The release package sets `doCheck = false`; the dedicated `nextest` check owns
  test execution and supplies `LATTICE_CAIRN_BIN`. The package check phase had
  compiled 335 library tests and 7 binary tests successfully, then failed on a
  lifecycle test that requires that environment.
- The candidate-worker tests resolve `chmod` and `sleep` through the host PATH
  instead of hardcoding `/run/current-system/sw/bin`, so they run in the Nix
  sandbox.

## Baseline and fixture refresh

The Octet baseline was produced by an older workspace/profile cohort
(rustc 1.96.0-nightly, 2026-03-21), so every accepted finding identity was stale.
The migration was reviewed as a delta against the pre-change HEAD rather than as
new code debt:

- HEAD with the current tool: 1471 findings.
- This release tree: 1453 findings.
- 21 findings removed (the compose move and the candidate-worker edit).
- 2 count/message drifts only (`module_file_count` 23→22 and
  `excessive_file_length` 1318→1322).
- No new semantic finding category.

The root baseline was regenerated to 1453 findings and the core baseline to 25
findings with the same tool/profile. `octet-precommit-check` then passed.

Traceability fixtures were refreshed for current reviewed sources:
`traceability-repository-validation-completion-reviewed.json`,
`traceability-scheduling-audit-reviewed.json`,
`traceability-scheduling-completion-reviewed.json`,
`traceability-remaining-credentials-reviewed.json`,
`traceability-webhook-auth-completion-reviewed.json`, the TUI promotion binding
counts, and the trybuild `compile-fail.stderr` fixture. All refreshed tests pass.

## Scoped TUI gap

Three TUI authoring PTY tests fail against the current TUI implementation:

- `operator_e2e_tui_process_authors_http_and_recovers_invalid_constrained_draft`
- `operator_e2e_tui_process_rejects_invalid_workflow_before_effect_and_remains_navigable`
- `operator_e2e_tui_process_resize_preserves_modal_workflow_and_keyboard_input`

They are ignored by the release gate with an explicit reason and remain tracked
by `complete-tui-nickel-authoring-parity`. The server release does not claim TUI
authoring-parity completion. This is a recorded limitation, not a passing test
result.

## Active lifecycle changes

The repository still contains open lifecycle changes. They are not silently
treated as released capability. The release owner must decide whether to finish,
archive, or explicitly defer each one before tagging.

| Change | Tasks | Default server impact | Release decision |
|---|---:|---|---|
| `add-native-bounded-http-transport` | 7/8 | HTTP effects | Finish coordinated checks or defer the change. |
| `add-wasm-component-operation-pack-runtime` | 0/15 | Optional operation-pack runtime | Not part of the default server surface; defer or finish. |
| `adopt-animus-secretspec-sops-providers` | 4/17 | Optional `animus-secretspec` feature | Not default; retain as an authorized optional feature. |
| `adopt-owned-source-agpl` | 2/3 | Licensing | Finish final verification before publishing. |
| `complete-tui-nickel-authoring-parity` | 3/18 | TUI authoring | Finish or state the TUI authoring boundary. |
| `compose-surface-pane-binding` | 9/10 | Separate `compose/` workspace | Finish verification with Tile access; not public surface. |
| `fix-embedded-nickel-imported-json-merge` | 4/7 | Nickel import | Finish remaining verification. |
| `fix-observed-workflow-exchange-cancellation` | 4/4 | Exchange cancellation | Archive through the normal lifecycle closeout. |
| `integrate-consumer-release-readiness` | 2/4 | Release assessment | Finish the consumer selection and replay tasks. |
| `orchestrate-asynchronous-mantle-checks` | 0/31 | Asynchronous CI | Not default; defer or finish. |
| `qualify-pure-inet-worker-profile` | 3/18 | Worker profile | Finish or state the qualification boundary. |

## External actions not authorized

- creating or pushing a release tag;
- publishing a GitHub release or binary archive;
- deploying the systemd unit or starting the exchange service;
- publishing or vendoring Tile source;
- enabling a hosted or managed offering.

## Non-claims

Local acquisition, metadata, policy, and lifecycle gates do not prove workflow
correctness, host confinement, external effect success, multi-tenant isolation,
deployment success, or production fitness.
