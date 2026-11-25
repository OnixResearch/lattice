## ADDED Requirements

### Requirement: Accepted Animus SecretSpec dependency

r[lattice.credentials.animus_secret_providers.dependency] Lattice MUST adopt SecretSpec support only from an immutable Animus revision whose accepted contract pins the reviewed SecretSpec Rust SDK, bounds provider execution, preserves feature isolation, and passes positive and negative conformance, provenance, license, and redaction evidence. Workspace-relative source and unarchived proposed APIs MUST NOT satisfy the dependency.

#### Scenario: Accepted cohort is pinned

r[lattice.credentials.animus_secret_providers.dependency.scenario.accepted]
- GIVEN Animus has archived its SecretSpec host change, synced accepted requirements, published an immutable revision, and passed required conformance rails
- WHEN Lattice syncs the dependency through its repository-owned vendor path
- THEN Lattice MUST bind the exact Animus revision, selected features, upstream SecretSpec version, source identities, and licenses
- AND the vendored bytes MUST match the recorded provenance.

#### Scenario: Proposed or stale cohort remains blocked

r[lattice.credentials.animus_secret_providers.dependency.scenario.blocked]
- GIVEN the Animus change is active, accepted markers or evidence are missing, the revision is mutable or stale, or the required feature or API differs
- WHEN Lattice dependency admission runs
- THEN adoption MUST remain blocked with the exact missing condition
- AND Lattice MUST NOT copy sibling source or fabricate compatibility locally.

### Requirement: Typed SecretSpec provider configuration

r[lattice.credentials.animus_secret_providers.config] Lattice MUST configure `animus-secretspec` as one typed Nickel provider variant with a unique id, explicit manifest identity, profile, scope, declaration mapping, allowed provider schemes, reason policy, and positive named request and value bounds without plaintext, provider credentials, encrypted payloads, host executable paths, or decrypted-file settings.

#### Scenario: Valid provider configuration projects

r[lattice.credentials.animus_secret_providers.config.scenario.valid]
- GIVEN an entry contains complete supported identities, profile, scope, declaration mapping, allowed environment, systemd credential, or SOPS scheme, reason policy, and limits
- WHEN Nickel validation and runtime projection run
- THEN Lattice MUST produce deterministic configuration for the selected pinned Animus SecretSpec host
- AND existing provider variants MUST remain unchanged.

#### Scenario: Unsafe provider configuration is rejected

r[lattice.credentials.animus_secret_providers.config.scenario.invalid]
- GIVEN an entry contains plaintext, provider credentials, duplicate identity, unknown kind, unsafe metadata, unrestricted scheme fallback, unsupported cohort, missing profile, scope, reason, or bound, or zero or excessive bound
- WHEN configuration validation runs
- THEN Lattice MUST reject it before opening the store, loading a workflow, resolving a credential, or invoking an effect.

### Requirement: Pure Lattice-to-Animus request adaptation

r[lattice.credentials.animus_secret_providers.request] Lattice MUST map an opaque workflow `SecretRef`, configured provider, profile, scope, provider allowlist, node or trigger context, safe reason, destination, and explicit capability facts into the accepted Animus SecretSpec request through a pure deterministic adapter while retaining Lattice effect and runtime admission authority.

#### Scenario: Reference maps to an admitted request

r[lattice.credentials.animus_secret_providers.request.scenario.admitted]
- GIVEN a workflow reference targets a configured compatible SecretSpec provider and the current node or trigger context admits its destination
- WHEN pure request adaptation runs
- THEN it MUST preserve provider, declaration, optional field, binding, profile, scope, allowlist, reason, policy, and bound identities in one Animus request
- AND it MUST perform no provider, file, process, network, environment, clock, store, or UI operation.

#### Scenario: Invalid mapping blocks resolution

r[lattice.credentials.animus_secret_providers.request.scenario.rejected]
- GIVEN the provider is missing, incompatible, malformed, denied for the destination, lacks a required reason or capability, selects a disallowed provider scheme, or would require ambient fallback
- WHEN request adaptation runs
- THEN it MUST return a stable credential diagnostic before invoking Animus or any dependent effect.

### Requirement: Per-binding ephemeral resolution

r[lattice.credentials.animus_secret_providers.resolve] Lattice MUST resolve only the credential bindings required by the current admitted node or trigger through the injected Animus SecretSpec host and MUST deliver each value directly to the existing execution boundary without persisting plaintext, provider credentials, or decrypted documents.

#### Scenario: Node binding resolves

r[lattice.credentials.animus_secret_providers.resolve.scenario.node]
- GIVEN an admitted shell, HTTP, agent, or other credential-consuming node references a configured SecretSpec declaration under an allowed provider scheme
- WHEN execution preparation resolves that binding successfully
- THEN Lattice MUST supply the ephemeral value only to that node adapter for the current attempt
- AND it MUST NOT place the value in workflow state, runner plans, caches, events, diagnostics, run details, replay, or store records.

#### Scenario: Trigger binding resolves

r[lattice.credentials.animus_secret_providers.resolve.scenario.trigger]
- GIVEN an admitted webhook or scheduled trigger requires a configured SecretSpec-backed credential
- WHEN trigger admission resolves that binding successfully
- THEN Lattice MUST use the value only at the existing authentication or invocation boundary
- AND it MUST retain only safe reference and outcome metadata.

#### Scenario: Resolution failure blocks effects

r[lattice.credentials.animus_secret_providers.resolve.scenario.failure]
- GIVEN resolution reports missing declaration, unsupported scheme, denied scope, unavailable provider, malformed input, missing executable, timeout, cancellation, oversized value, cleanup uncertainty, or another non-completed outcome
- WHEN Lattice prepares the dependent node or trigger
- THEN it MUST return a fixed safe credential failure and MUST NOT invoke the dependent effect or an ambient fallback provider.

### Requirement: SecretSpec diagnostics and evidence are value free

r[lattice.credentials.animus_secret_providers.redaction] Provider validation, reachability, execution, trigger, cache, history, replay, diagnostic, and mapping-report surfaces MUST omit resolved values, value-derived hashes, decrypted documents, private keys, provider credentials, full host paths, stdout, stderr, and arbitrary backend text.

#### Scenario: Safe success evidence is rendered

r[lattice.credentials.animus_secret_providers.redaction.scenario.success]
- GIVEN a SecretSpec binding resolves and the dependent effect executes
- WHEN user-visible or persisted evidence is produced
- THEN Lattice MAY show safe provider, declaration, profile, scope, scheme, reason, destination, bound, and fixed outcome metadata
- AND it MUST NOT show or hash the resolved value.

#### Scenario: Hostile failure text is discarded

r[lattice.credentials.animus_secret_providers.redaction.scenario.failure]
- GIVEN Animus, SecretSpec, a provider, worker, stdout, or stderr supplies failure text containing secret sentinels or host details
- WHEN Lattice maps and renders the failure
- THEN it MUST discard the arbitrary text and emit only a fixed local diagnostic class with safe opaque context.

### Requirement: Credential references round trip across node kinds

r[lattice.credentials.animus_secret_providers.round_trip] Lattice MUST serialize workflow credential references independently of node configuration kind so every admitted built-in node kind preserves provider, declaration, optional field, and binding identities across TUI save or load and Nickel export or import without serializing resolved values.

#### Scenario: Cross-kind credential round trip succeeds

r[lattice.credentials.animus_secret_providers.round_trip.scenario.preserve]
- GIVEN a workflow attaches valid opaque credential references to supported shell, HTTP, file, image, agent, condition, merge, or subworkflow nodes
- WHEN the workflow is saved, loaded, exported to Nickel, and imported
- THEN each admitted reference MUST remain attached to the same node and binding with equivalent identity
- AND no resolved material MAY appear in either persistence format.

#### Scenario: Deleted node leaves no orphan credential

r[lattice.credentials.animus_secret_providers.round_trip.scenario.delete]
- GIVEN a TUI-authored node has credential references and other per-node state
- WHEN the operator deletes that node
- THEN one workflow lifecycle mutation MUST remove the graph node, config, credentials, runner settings, and incident edges
- AND subsequent validation and Nickel export or import MUST contain no orphan state.

### Requirement: Safe provider TUI and CLI behavior

r[lattice.credentials.animus_secret_providers.tui] Lattice SHOULD expose SecretSpec provider selection, reference editing, validation, and reachability through safe TUI and CLI surfaces while preserving prior values on invalid edits and never displaying secret material.

#### Scenario: Provider reference edit remains recoverable

r[lattice.credentials.animus_secret_providers.tui.scenario.edit]
- GIVEN a selected node credential binding is editable
- WHEN the operator selects a configured provider and enters a valid declaration or rejects an invalid one
- THEN the TUI MUST store only the valid opaque reference, preserve the prior reference on failure, and remain usable.

#### Scenario: Reachability does not disclose a value

r[lattice.credentials.animus_secret_providers.tui.scenario.reachability]
- GIVEN a configured provider is tested from TUI or CLI
- WHEN the safe probe completes or fails
- THEN the surface MUST show only a fixed configured, reachable, missing, denied, unavailable, timed-out, or unsupported class supported by typed evidence
- AND it MUST NOT render a value or arbitrary backend output.

### Requirement: Existing credential providers remain compatible

r[lattice.credentials.animus_secret_providers.compatibility] Existing environment, local-vault, command, workflow, trigger, cache, redaction, and disabled-feature behavior MUST remain unchanged when `animus-secretspec` is not configured.

#### Scenario: Optional provider is absent

r[lattice.credentials.animus_secret_providers.compatibility.scenario.absent]
- GIVEN configuration contains no `animus-secretspec` entry
- WHEN Lattice validates, opens workflows, and executes existing provider paths
- THEN it MUST perform no Animus secret-host, SecretSpec, manifest, provider-credential, encrypted-file, executable, or new network operation.

### Requirement: Secret provider documentation preserves boundaries

r[lattice.credentials.animus_secret_providers.documentation] Operator documentation MUST describe explicit SecretSpec manifest, profile, scope, provider-scheme, declaration, key-authority, reason, audit, migration, rollback, and optional-feature boundaries without example plaintext secrets.

#### Scenario: Documentation does not overclaim

r[lattice.credentials.animus_secret_providers.documentation.scenario.non_claim]
- GIVEN provider setup and successful fixture evidence are documented
- WHEN claim validation reviews the text
- THEN it MUST NOT claim upstream cryptographic correctness, global authorization, value correctness or freshness, arbitrary confidentiality, audit completeness, provider truth, external exactly-once effects, release eligibility, or production readiness.

### Requirement: Secret provider verification includes failures

r[lattice.credentials.animus_secret_providers.verification] Adoption MUST pair positive SecretSpec environment, systemd credential, and SOPS node and trigger fixtures with negative dependency, config, mapping, capability, scope, provider, worker, redaction, round-trip, deletion, persistence, cache, history or replay, disabled-feature, and provenance fixtures.

#### Scenario: Closeout evidence is complete

r[lattice.credentials.animus_secret_providers.verification.scenario.closeout]
- GIVEN maintainers intend to archive the provider adoption
- WHEN closeout validation runs
- THEN evidence MUST name the accepted Animus revision and provenance, pinned SecretSpec cohort, focused positive and negative tests, repository checks, Octet result, Cairn validation and gates, and selected Nix checks
- AND every skipped rail MUST include an exact blocker and next-best check.
