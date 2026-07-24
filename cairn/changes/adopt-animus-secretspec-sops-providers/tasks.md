## Phase 1: Dependency and baseline

- [ ] [serial] Confirm Animus change `add-secretspec-sops-host-resolution` is archived, accepted requirements are synced, focused positive/negative conformance and feature-isolation rails pass, and one immutable revision exposes the reviewed contract; remain blocked rather than copying workspace source if any condition is missing. [depends:agent_harness_runtime.secret_resolution.verification]
- [ ] [serial] Establish and record focused baselines for credential registry resolution, command/local-vault/env providers, workflow Nickel credential round trips across node kinds, agent and webhook credential delivery, redaction, cache identity, history/replay, configuration validation, and Animus vendor provenance. r[lattice.credentials.animus_secret_providers.compatibility] r[lattice.credentials.animus_secret_providers.redaction]
- [ ] [serial] Sync the exact Animus revision through `scripts/sync_animus.rs`, record source/BLAKE3/license/feature provenance, and prove no workspace-relative runtime dependency or unrelated default/no-std feature expansion. r[lattice.credentials.animus_secret_providers.dependency]

## Phase 2: Typed configuration and pure adaptation

- [ ] [serial] Add typed Nickel `animus-secretspec` and `animus-sops` provider variants with unique ids, explicit manifest/encrypted roots and relative paths, project/profile/provider or format/key mapping fields, reason policy, exact executable/runtime identity, explicit credential environment-key names, and named positive count/byte/deadline limits. r[lattice.credentials.animus_secret_providers.config]
- [ ] [parallel] Add positive and negative config fixtures for both variants and unknown kinds, duplicate ids, plaintext values, unsafe paths, provider ambiguity, unrestricted environment, unsupported cohorts/formats, missing reason, and zero/excessive limits. r[lattice.credentials.animus_secret_providers.config.scenario.valid] r[lattice.credentials.animus_secret_providers.config.scenario.invalid]
- [ ] [serial] Implement a pure adapter from Lattice `SecretRef`, provider config, node/trigger context, reason, and explicit capability facts into the pinned Animus request, preserving Lattice provider/key/field/binding identity and effect admission. r[lattice.credentials.animus_secret_providers.request]
- [ ] [parallel] Add positive and negative pure tests for SecretSpec and SOPS request mapping, optional field/key-path behavior, missing registry entries, unsupported capabilities/cohorts, wrong context, unsafe destination, missing reason, and no ambient fallback. r[lattice.credentials.animus_secret_providers.request.scenario.admitted] r[lattice.credentials.animus_secret_providers.request.scenario.rejected]

## Phase 3: Provider shells and execution boundary

- [ ] [serial] Add thin Lattice credential-provider wrappers over injected pinned Animus SecretSpec and SOPS host services, resolving only current node/trigger bindings and delivering ephemeral material directly to existing adapter-specific execution preparation. r[lattice.credentials.animus_secret_providers.resolve]
- [ ] [serial] Map typed Animus outcomes into fixed Lattice provider diagnostics and safe reachability results while discarding arbitrary SDK/provider/stdout/stderr text and preserving no plaintext, decrypted document, full path, credential environment value, or value-derived hash. r[lattice.credentials.animus_secret_providers.redaction]
- [ ] [parallel] Add fake-host positive and negative tests for shell, HTTP, agent, webhook, missing key/file/executable/credential, denial, timeout, cancellation, truncation, malformed/duplicate output, unsafe paths, hostile error sentinels, dependent-effect blocking, and material drop. r[lattice.credentials.animus_secret_providers.resolve.scenario.node] r[lattice.credentials.animus_secret_providers.resolve.scenario.trigger] r[lattice.credentials.animus_secret_providers.resolve.scenario.failure]

## Phase 4: Workflow/TUI parity and evidence

- [x] [serial] Refactor workflow Nickel export/import so credentials are serialized from the workflow map independently of node config variant and prove every admitted built-in node kind preserves opaque refs; do not serialize resolved values. r[lattice.credentials.animus_secret_providers.round_trip]
- [ ] [serial] Route TUI credential editing, provider selection/reachability, node deletion, save/load, and diagnostics through workflow-level lifecycle operations so refs remain valid or are removed atomically with their node. r[lattice.credentials.animus_secret_providers.tui]
- [ ] [parallel] Add hostile-sentinel tests across Nickel, redb, runner plans, cache keys, events, output, diagnostics, run details, replay, trigger records, reachability rendering, and mapping reports; pair positive delivery with negative leak cases. r[lattice.credentials.animus_secret_providers.redaction] r[lattice.credentials.animus_secret_providers.round_trip]
- [ ] [parallel] Preserve and rerun existing environment, local-vault, command-provider, no-new-config, and disabled-feature behavior. r[lattice.credentials.animus_secret_providers.compatibility]

## Phase 5: Completion

- [ ] [serial] Update provider documentation and README references with SecretSpec/SOPS setup boundaries, explicit provider configuration, key authority, reason/audit limits, migration steps, and non-claims without example plaintext secrets. r[lattice.credentials.animus_secret_providers.documentation]
- [ ] [serial] Run focused credential/workflow/TUI/agent/webhook/history/replay tests, `scripts/check.rs`, Animus provenance checks, `octet-precommit-check`, Cairn validation and proposal/design/tasks gates, and bounded `nix flake check -L --option secret-key-files ''`; preserve exact evidence or blockers. r[lattice.credentials.animus_secret_providers.verification]
- [ ] [serial] Sync accepted credential requirements, verify markers reached `cairn/specs/credentials/spec.md`, and archive only after the immutable Animus dependency and all selected Lattice checks pass. r[lattice.credentials.animus_secret_providers.verification]
