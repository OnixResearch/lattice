## Why

Lattice already stores opaque credential references and resolves them through an explicit provider registry, but operators cannot select SecretSpec declarations or SOPS-encrypted files as first-class providers. Product-local implementations would duplicate provider admission, process bounds, redaction, and evidence that belong in the reusable Animus secret-resolution boundary.

This is a follow-up to Animus change `add-secretspec-sops-host-resolution`. It must remain blocked until Animus publishes an immutable revision with the authority-free request contract, optional SecretSpec and direct SOPS host adapters, safe receipt behavior, and passing compatibility evidence. Workspace-relative source is not an acceptable runtime dependency.

## What Changes

- Pin and vendor the accepted Animus secret-resolution cohort through Lattice's repository-owned sync path and verify its provenance, feature isolation, licenses, and exact public contract.
- Add typed Nickel configuration for `animus-secretspec` and `animus-sops` credential-provider entries with explicit provider ids, manifest/profile/provider or encrypted-root/file/key settings, reason policy, allowed credential environment-key names, and positive count/byte/deadline bounds.
- Adapt Lattice `SecretRef` identities to Animus resolution requests while retaining Lattice ownership of workflow references, provider registry admission, cache identity, runtime preparation, redaction, history, replay, triggers, and effect gating.
- Deliver one resolved value directly to the existing node/trigger execution boundary without storing plaintext or a value-derived hash in workflows, Nickel, redb, runner plans, events, diagnostics, run records, replay, caches, or mapping reports.
- Expose stable provider validation and reachability results in CLI/TUI without loading a secret merely for display and without rendering arbitrary Animus, SecretSpec, SOPS, provider, stdout, or stderr text.
- Preserve credential references for every node kind across TUI save and Nickel export/import so SecretSpec/SOPS-backed agent, HTTP, webhook, and other references cannot disappear during authoring round trips.
- Keep absent features backward compatible: existing environment, local-vault, and command providers retain their behavior and no SecretSpec/SOPS dependency, executable, file, provider client, or network effect becomes ambient.

## Success Criteria

- A valid workflow reference resolves through a configured SecretSpec provider and through a separately configured direct SOPS provider using the pinned Animus contract.
- Unknown provider ids, malformed references, unsupported Animus/provider cohorts, missing profiles/keys/files/executables/credentials, path escape, timeout, truncation, malformed/duplicate output, provider denial, and unsafe diagnostics fail before node or trigger effects.
- Workflow and store serialization contain provider/key/field/binding identities only; agent and every other node kind preserve those references through TUI save and Nickel round trips.
- Safe reachability, run, history, replay, trigger, cache, and diagnostic tests include hostile plaintext sentinels and prove value omission.
- Existing providers and workflows pass unchanged when the new optional provider kinds are absent.

## Out of Scope

- Reimplementing SecretSpec or SOPS, managing KMS/age/PGP keys, provider login, migration/import, secret generation, rotation, revocation, or global access policy.
- Treating SecretSpec's local audit as durable Lattice run evidence or treating SOPS encryption as an application authorization interface.
- Automatic provider selection outside the explicit registry, ambient manifest/file discovery, whole-profile environment injection, or storing decrypted documents.
- Claiming secret correctness/freshness, arbitrary confidentiality, provider truth, external effect uniqueness, release eligibility, or production readiness.

## Impact

- **Dependency:** hard lifecycle dependency on accepted Animus secret-resolution requirements and an immutable synced revision.
- **Configuration:** additive typed Nickel provider variants; existing configuration remains valid.
- **Runtime:** optional provider shells feed the existing credential preparation boundary; Lattice retains product authority and evidence semantics.
- **Testing:** focused positive and negative provider, workflow-format, TUI, trigger, cache, history/replay, provenance, Octet, Cairn, and Nix checks.
