## Phase 1: Dependency and baseline

- [x] [serial] Confirm Animus change `add-secretspec-sops-host-resolution` is archived, accepted requirements are synced, bounded provider and redaction rails pass, and one immutable revision exposes the SecretSpec `v0.17.0` host contract; remain blocked rather than copying workspace source if any condition is missing. [depends:credentials.secretspec.provider_conformance]
- [ ] [serial] Establish and record focused baselines for credential registry resolution, command/local-vault/env providers, workflow Nickel credential round trips across node kinds, agent and webhook credential delivery, redaction, cache identity, history/replay, configuration validation, and Animus vendor provenance. r[lattice.credentials.animus_secret_providers.compatibility] r[lattice.credentials.animus_secret_providers.redaction]
- [ ] [serial] Sync the exact Animus revision through `scripts/sync_animus.rs`, record Animus and SecretSpec source/BLAKE3/license/feature provenance, and prove no workspace-relative runtime dependency or unrelated default or `no_std` feature expansion. r[lattice.credentials.animus_secret_providers.dependency]

## Phase 2: Typed configuration and pure adaptation

- [ ] [serial] Add one typed Nickel `animus-secretspec` provider variant with a unique id, explicit manifest identity, profile, scope, declaration mapping, allowed provider schemes, reason policy, and named positive request and value bounds. r[lattice.credentials.animus_secret_providers.config]
- [ ] [parallel] Add positive and negative config fixtures for allowed environment, systemd credential, and SOPS schemes plus unknown kinds, duplicate ids, plaintext values, unsafe metadata, provider ambiguity, unsupported cohorts, missing profile or scope, missing reason, and zero or excessive bounds. r[lattice.credentials.animus_secret_providers.config.scenario.valid] r[lattice.credentials.animus_secret_providers.config.scenario.invalid]
- [x] [serial] Implement a pure adapter from Lattice `SecretRef`, provider config, node or trigger context, reason, destination, and explicit capability facts into the pinned Animus request. r[lattice.credentials.animus_secret_providers.request]
- [ ] [parallel] Add positive and negative pure tests for request mapping, optional fields, missing registry entries, unsupported cohorts, disallowed provider schemes, wrong context, unsafe destination, missing reason, and no ambient fallback. r[lattice.credentials.animus_secret_providers.request.scenario.admitted] r[lattice.credentials.animus_secret_providers.request.scenario.rejected]

## Phase 3: Provider shell and execution boundary

- [ ] [serial] Add a thin Lattice credential-provider wrapper over the injected pinned Animus SecretSpec host, resolving only current node or trigger bindings and delivering ephemeral material directly to existing execution preparation. r[lattice.credentials.animus_secret_providers.resolve]
- [x] [serial] Map typed Animus outcomes into fixed Lattice diagnostics and safe reachability results while discarding arbitrary SDK, provider, worker, stdout, and stderr text. Preserve no plaintext, decrypted document, full path, provider credential, or value-derived hash. r[lattice.credentials.animus_secret_providers.redaction]
- [ ] [parallel] Add fake-host positive and negative tests for shell, HTTP, agent, webhook, environment, systemd credential, SOPS, missing declaration or executable, provider denial, timeout, cancellation, oversized value, unsafe policy, hostile error sentinels, dependent-effect blocking, and material drop. r[lattice.credentials.animus_secret_providers.resolve.scenario.node] r[lattice.credentials.animus_secret_providers.resolve.scenario.trigger] r[lattice.credentials.animus_secret_providers.resolve.scenario.failure]

## Phase 4: Workflow/TUI parity and evidence

- [x] [serial] Refactor workflow Nickel export/import so credentials are serialized from the workflow map independently of node config variant and prove every admitted built-in node kind preserves opaque refs; do not serialize resolved values. r[lattice.credentials.animus_secret_providers.round_trip]
- [ ] [serial] Route TUI credential editing, provider selection/reachability, node deletion, save/load, and diagnostics through workflow-level lifecycle operations so refs remain valid or are removed atomically with their node. r[lattice.credentials.animus_secret_providers.tui]
- [ ] [parallel] Add hostile-sentinel tests across Nickel, redb, runner plans, cache keys, events, output, diagnostics, run details, replay, trigger records, reachability rendering, and mapping reports; pair positive delivery with negative leak cases. r[lattice.credentials.animus_secret_providers.redaction] r[lattice.credentials.animus_secret_providers.round_trip]
- [ ] [parallel] Preserve and rerun existing environment, local-vault, command-provider, no-new-config, and disabled-feature behavior. r[lattice.credentials.animus_secret_providers.compatibility]

## Phase 5: Completion

- [ ] [serial] Update provider documentation and README references with SecretSpec setup, profile, scope, provider-scheme, deployment, reason/audit, migration, rollback, and non-claim boundaries without example plaintext secrets. r[lattice.credentials.animus_secret_providers.documentation]
- [ ] [serial] Run focused credential/workflow/TUI/agent/webhook/history/replay tests, `scripts/check.rs`, Animus provenance checks, `octet-precommit-check`, Cairn validation and proposal/design/tasks gates, and bounded `nix flake check -L --option secret-key-files ''`; preserve exact evidence or blockers. r[lattice.credentials.animus_secret_providers.verification]
- [ ] [serial] Sync accepted credential requirements, verify markers reached `.cairn/specs/credentials/spec.md`, and archive only after the immutable Animus dependency and all selected Lattice checks pass. r[lattice.credentials.animus_secret_providers.verification]
