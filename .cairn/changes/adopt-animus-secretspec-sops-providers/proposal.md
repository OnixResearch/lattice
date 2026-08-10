## Why

Lattice already stores opaque credential references and resolves them through an explicit provider registry. Operators still cannot select SecretSpec declarations as a first-class provider. A product-local implementation would duplicate profile, scope, provider, process-bound, redaction, and evidence rules that belong in the reusable Animus host boundary.

SecretSpec `v0.17.0` now provides the Rust SDK and a native SOPS provider. Lattice should consume one reviewed Animus SecretSpec adapter. It should not create a second direct SOPS provider or expose encrypted-file mechanics in workflow state.

This change follows Animus change `add-secretspec-sops-host-resolution`. It remains blocked until Animus publishes an immutable revision with accepted requirements, the pinned SecretSpec SDK, bounded provider execution, safe receipts, and passing conformance evidence. Workspace-relative source is not an acceptable runtime dependency.

## What Changes

- Pin and vendor the accepted Animus secret-resolution cohort through Lattice's repository-owned sync path. Verify provenance, feature isolation, licenses, SecretSpec version, and the exact public contract.
- Add typed Nickel configuration for one `animus-secretspec` credential-provider kind with explicit provider id, manifest identity, profile, scope, allowed provider schemes, reason policy, and positive request and value bounds.
- Adapt Lattice `SecretRef` identities to Animus resolution requests while retaining Lattice ownership of workflow references, registry admission, cache identity, runtime preparation, redaction, history, replay, triggers, and effect gating.
- Let Animus and the SecretSpec Rust SDK select environment, systemd credential, SOPS, or another reviewed provider. Keep provider credentials and SOPS execution details outside Lattice workflows.
- Deliver one resolved value directly to the existing node or trigger boundary without storing plaintext or a value-derived hash in workflows, Nickel, redb, runner plans, events, diagnostics, records, replay, caches, or reports.
- Expose stable provider validation and reachability results without loading a secret for display or rendering arbitrary Animus, SecretSpec, provider, stdout, or stderr text.
- Preserve credential references for every node kind across TUI save and Nickel export or import.
- Keep absent features backward compatible. Existing providers retain their behavior, and no SecretSpec dependency or provider effect becomes ambient.

## Success Criteria

- A valid workflow reference resolves through a configured SecretSpec declaration using an allowed environment, systemd credential, or SOPS profile from the pinned Animus host.
- Unknown provider ids, malformed references, unsupported cohorts, missing profiles, scopes, declarations, executables, or provider credentials, unsafe provider policy, timeout, cancellation, truncation, provider denial, and unsafe diagnostics fail before node or trigger effects.
- Workflow and store serialization contain provider, declaration, field, and binding identities only. Every admitted node kind preserves those references through TUI save and Nickel round trips.
- Safe reachability, run, history, replay, trigger, cache, and diagnostic tests include hostile plaintext sentinels and prove value omission.
- Existing providers and workflows pass unchanged when `animus-secretspec` is absent.

## Out of Scope

- Reimplementing SecretSpec, adding a direct SOPS adapter, managing KMS, age, or PGP keys, provider login, secret generation, rotation, revocation, or global access policy.
- Treating SecretSpec audit output as durable Lattice run evidence or treating SOPS encryption as application authorization.
- Automatic provider selection outside explicit manifest, profile, scope, and allowlist policy; ambient manifest discovery; whole-profile environment injection; or decrypted-document storage.
- Claiming secret correctness or freshness, arbitrary confidentiality, provider truth, external effect uniqueness, release eligibility, or production readiness.

## Impact

- **Dependency:** hard lifecycle dependency on accepted Animus secret-resolution requirements and an immutable synced revision.
- **Configuration:** one additive typed Nickel provider variant; existing configuration remains valid.
- **Runtime:** the optional provider shell feeds the existing credential preparation boundary; Lattice retains product authority and evidence semantics.
- **Testing:** focused positive and negative provider, workflow-format, TUI, trigger, cache, history or replay, provenance, Octet, Cairn, and Nix checks.
