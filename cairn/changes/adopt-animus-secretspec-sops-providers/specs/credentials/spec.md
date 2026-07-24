## ADDED Requirements

### Requirement: Accepted Animus secret-resolution dependency

r[lattice.credentials.animus_secret_providers.dependency] Lattice MUST adopt SecretSpec and SOPS support only from an immutable Animus revision whose accepted secret-resolution contract, optional host adapters, positive and negative conformance, feature isolation, provenance, and license evidence have passed; workspace-relative source and unarchived proposed APIs MUST NOT satisfy the dependency.

#### Scenario: Accepted cohort is pinned

r[lattice.credentials.animus_secret_providers.dependency.scenario.accepted]
- GIVEN Animus has archived its secret-resolution change, synced accepted requirements, published a consumable immutable revision, and passed required conformance rails
- WHEN Lattice syncs the dependency through its repository-owned vendor path
- THEN Lattice MUST bind the exact revision, selected features, source identities, and licenses
- AND the vendored bytes MUST match the recorded provenance.

#### Scenario: Proposed or stale cohort remains blocked

r[lattice.credentials.animus_secret_providers.dependency.scenario.blocked]
- GIVEN the Animus change is active, its accepted markers or evidence are missing, the revision is mutable or stale, or the required feature/API differs
- WHEN Lattice dependency admission runs
- THEN adoption MUST remain blocked with the exact missing condition
- AND Lattice MUST NOT copy sibling source or fabricate compatibility locally.

### Requirement: Typed SecretSpec and SOPS provider configuration

r[lattice.credentials.animus_secret_providers.config] Lattice MUST configure `animus-secretspec` and `animus-sops` credential providers as distinct typed Nickel variants with unique ids, explicit safe roots and relative paths, declaration/provider or encrypted-document settings, reason policy, allowed credential environment-key names, exact runtime/executable identity where applicable, and positive named bounds without plaintext values.

#### Scenario: Valid provider configurations project

r[lattice.credentials.animus_secret_providers.config.scenario.valid]
- GIVEN SecretSpec and SOPS entries contain complete supported identities, capabilities, safe paths, reasons, and limits
- WHEN Nickel validation and runtime projection run
- THEN Lattice MUST produce deterministic provider configs for the selected pinned Animus host adapters
- AND existing provider variants MUST remain unchanged.

#### Scenario: Unsafe provider configuration is rejected

r[lattice.credentials.animus_secret_providers.config.scenario.invalid]
- GIVEN a provider entry contains plaintext material, duplicate identity, unknown kind, unsafe path, unrestricted inherited environment, ambiguous executable, unsupported format/cohort, missing reason, or zero or excessive bound
- WHEN configuration validation runs
- THEN Lattice MUST reject it before opening the store, loading a workflow, resolving a credential, or invoking an effect.

### Requirement: Pure Lattice-to-Animus request adaptation

r[lattice.credentials.animus_secret_providers.request] Lattice MUST map an opaque workflow `SecretRef`, configured provider, node or trigger context, safe reason, destination, and explicit capability facts into the accepted Animus secret-resolution request through a pure deterministic adapter while retaining Lattice effect and runtime admission authority.

#### Scenario: Reference maps to an admitted request

r[lattice.credentials.animus_secret_providers.request.scenario.admitted]
- GIVEN a workflow reference targets a configured compatible SecretSpec or SOPS provider and the current node or trigger context admits its destination
- WHEN pure request adaptation runs
- THEN it MUST preserve provider, key, optional field or lexical key-path, binding, reason, policy, and bound identities in one Animus request
- AND it MUST perform no provider, file, process, network, environment, clock, store, or UI operation.

#### Scenario: Invalid mapping blocks resolution

r[lattice.credentials.animus_secret_providers.request.scenario.rejected]
- GIVEN the provider is missing, incompatible, malformed, denied for the destination, lacks a required reason/capability, or would require ambient fallback
- WHEN request adaptation runs
- THEN it MUST return a stable credential diagnostic before invoking Animus or any dependent effect.

### Requirement: Per-binding ephemeral resolution

r[lattice.credentials.animus_secret_providers.resolve] Lattice MUST resolve only the credential bindings required by the current admitted node or trigger through the injected Animus host service and MUST deliver each value directly to the existing execution boundary without persisting plaintext or decrypted documents.

#### Scenario: Node binding resolves

r[lattice.credentials.animus_secret_providers.resolve.scenario.node]
- GIVEN an admitted shell, HTTP, agent, or other credential-consuming node references a configured SecretSpec or SOPS provider
- WHEN execution preparation resolves that binding successfully
- THEN Lattice MUST supply the ephemeral value only to that node adapter for the current attempt
- AND it MUST NOT place the value in workflow state, runner plans, caches, events, diagnostics, run details, replay, or store records.

#### Scenario: Trigger binding resolves

r[lattice.credentials.animus_secret_providers.resolve.scenario.trigger]
- GIVEN an admitted webhook or scheduled trigger requires a configured SecretSpec or SOPS-backed credential
- WHEN trigger admission resolves that binding successfully
- THEN Lattice MUST use the value only at the existing authentication or invocation boundary
- AND it MUST retain only safe reference and outcome metadata.

#### Scenario: Resolution failure blocks effects

r[lattice.credentials.animus_secret_providers.resolve.scenario.failure]
- GIVEN resolution reports missing, unsupported, denied, unavailable, unsafe-path, malformed, duplicate, timeout, cancellation, truncation, nonzero, cleanup-uncertain, or other non-completed outcome
- WHEN Lattice prepares the dependent node or trigger
- THEN it MUST return a fixed safe credential failure and MUST NOT invoke the dependent effect or an ambient fallback provider.

### Requirement: SecretSpec/SOPS diagnostics and evidence are value free

r[lattice.credentials.animus_secret_providers.redaction] Provider validation, reachability, execution, trigger, cache, history, replay, diagnostic, and mapping-report surfaces MUST omit resolved values, value-derived hashes, decrypted documents, private keys, provider credentials, credential environment values, full host paths, stdout, stderr, and arbitrary backend text.

#### Scenario: Safe success evidence is rendered

r[lattice.credentials.animus_secret_providers.redaction.scenario.success]
- GIVEN a SecretSpec or SOPS binding resolves and the dependent effect executes
- WHEN user-visible or persisted evidence is produced
- THEN Lattice MAY show safe provider/reference identities, reason/destination classes, configured bounds, and fixed outcome/redaction metadata
- AND it MUST NOT show or hash the resolved value.

#### Scenario: Hostile failure text is discarded

r[lattice.credentials.animus_secret_providers.redaction.scenario.failure]
- GIVEN Animus, SecretSpec, SOPS, a provider, stdout, or stderr supplies failure text containing secret sentinels or host details
- WHEN Lattice maps and renders the failure
- THEN it MUST discard the arbitrary text and emit only a fixed local diagnostic class with safe opaque context.

### Requirement: Credential references round trip across node kinds

r[lattice.credentials.animus_secret_providers.round_trip] Lattice MUST serialize workflow credential references independently of node configuration kind so every admitted built-in node kind preserves provider, key, optional field, and binding identities across TUI save/load and Nickel export/import without serializing resolved values.

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
- AND subsequent validation and Nickel export/import MUST contain no orphan state.

### Requirement: Safe provider TUI and CLI behavior

r[lattice.credentials.animus_secret_providers.tui] Lattice SHOULD expose SecretSpec/SOPS provider selection, reference editing, validation, and reachability through safe TUI and CLI surfaces while preserving prior values on invalid edits and never displaying secret material.

#### Scenario: Provider reference edit remains recoverable

r[lattice.credentials.animus_secret_providers.tui.scenario.edit]
- GIVEN a selected node credential binding is editable
- WHEN the operator selects a configured provider and enters a valid key or rejects an invalid one
- THEN the TUI MUST store only the valid opaque reference, preserve the prior reference on failure, and remain usable.

#### Scenario: Reachability does not disclose a value

r[lattice.credentials.animus_secret_providers.tui.scenario.reachability]
- GIVEN a configured provider is tested from TUI or CLI
- WHEN the safe probe completes or fails
- THEN the surface MUST show only a fixed configured/reachable/missing/denied/unavailable/timed-out/unsupported class supported by typed evidence
- AND it MUST NOT render a value or arbitrary backend output.

### Requirement: Existing credential providers remain compatible

r[lattice.credentials.animus_secret_providers.compatibility] Existing environment, local-vault, command, workflow, trigger, cache, redaction, and disabled-feature behavior MUST remain unchanged when Animus SecretSpec/SOPS providers are not configured.

#### Scenario: Optional providers are absent

r[lattice.credentials.animus_secret_providers.compatibility.scenario.absent]
- GIVEN configuration contains no `animus-secretspec` or `animus-sops` entry
- WHEN Lattice validates, opens workflows, and executes existing provider paths
- THEN it MUST perform no Animus secret-host, SecretSpec, SOPS, manifest, encrypted-file, provider-credential, executable, or new network operation.

### Requirement: Secret provider documentation preserves boundaries

r[lattice.credentials.animus_secret_providers.documentation] Operator documentation MUST describe explicit SecretSpec/SOPS configuration, provider/key authority, reason and audit limits, migration and rollback, optional feature behavior, and bounded non-claims without example plaintext secrets.

#### Scenario: Documentation does not overclaim

r[lattice.credentials.animus_secret_providers.documentation.scenario.non_claim]
- GIVEN provider setup and successful fixture evidence are documented
- WHEN claim validation reviews the text
- THEN it MUST NOT claim upstream cryptographic correctness, global authorization, value correctness or freshness, arbitrary confidentiality, audit completeness, provider truth, external exactly-once effects, release eligibility, or production readiness.

### Requirement: Secret provider verification includes failures

r[lattice.credentials.animus_secret_providers.verification] Adoption MUST pair positive SecretSpec/SOPS node and trigger fixtures with negative dependency, config, mapping, capability, path, provider, process, parser, redaction, round-trip, deletion, persistence, cache, history/replay, disabled-feature, and provenance fixtures.

#### Scenario: Closeout evidence is complete

r[lattice.credentials.animus_secret_providers.verification.scenario.closeout]
- GIVEN maintainers intend to archive the provider adoption
- WHEN closeout validation runs
- THEN evidence MUST name the accepted Animus revision and provenance, focused positive and negative tests, repository checks, Octet result, Cairn validation/gates, and selected Nix checks
- AND every skipped rail MUST include an exact blocker and next-best check.
