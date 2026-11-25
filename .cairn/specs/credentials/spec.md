# Credentials Specification

## Purpose

Defines the `credentials` capability.

## Requirements

### Requirement: Secret reference model
r[n8nclone.credentials.secret_refs.model] The system MUST represent workflow credentials as opaque secret references instead of plaintext secret values and MUST reject plaintext credential fields through a stable redaction-safe credential diagnostic before store access.

#### Scenario: Store secret reference
r[n8nclone.credentials.secret_refs.model.store_ref]
- GIVEN a node resolves an execution-only credential and the workflow is subsequently saved or exported
- WHEN repository-owned store and Nickel export boundaries serialize the workflow
- THEN they store only provider, key, optional field, and binding identities and do not serialize the resolved plaintext value

#### Scenario: Reject inline secret field
r[n8nclone.credentials.secret_refs.model.reject_plaintext]
- GIVEN an imported workflow provides plaintext secret material in a credential field
- WHEN workflow import runs
- THEN the system returns a stable credential diagnostic, omits the plaintext value from diagnostics, and does not create or mutate the workflow store

### Requirement: Credential provider interface
r[n8nclone.credentials.provider_interface] The system MUST resolve secret references through a credential-provider interface at execution boundaries.

#### Scenario: Resolve secret for node execution
r[n8nclone.credentials.provider_interface.resolve]
- GIVEN a node references a credential that the configured provider can resolve
- WHEN the node is prepared for execution
- THEN the system supplies the resolved secret to the node adapter without storing it in the workflow

#### Scenario: Missing secret fails safely
r[n8nclone.credentials.provider_interface.missing]
- GIVEN a node references a credential that the configured provider cannot resolve
- WHEN the workflow runs
- THEN the system fails the node with a clear credential error and does not execute the side-effecting adapter

### Requirement: Secret redaction
r[n8nclone.credentials.redaction] The system MUST redact resolved secret material before user-visible output, tracing logs, run history, JSON summaries, or persisted run-detail surfaces can observe it.

#### Scenario: Redact run output
r[n8nclone.credentials.redaction.output]
- GIVEN a node emits a resolved secret or overlapping secret fragment through stdout and stderr
- WHEN execution output, tracing logs, run history, JSON summaries, and persisted run details are observed
- THEN every surface MUST omit the plaintext values, output/history/JSON/persisted details MUST preserve explicit redaction markers, and tracing logs MUST retain safe non-secret node context

#### Scenario: Redact validation message
r[n8nclone.credentials.redaction.validation]
- GIVEN a credential diagnostic mentions a secret reference and a separate plaintext value exists
- WHEN text and JSON diagnostics are rendered
- THEN both renderings MUST preserve the reference identity and MUST omit the plaintext value

### Requirement: Credential validation
r[n8nclone.credentials.validation] The system SHOULD validate secret-reference shape before execution.

#### Scenario: Malformed secret reference
r[n8nclone.credentials.validation.malformed]
- GIVEN a node contains a malformed secret reference
- WHEN workflow validation runs
- THEN the system reports a fatal diagnostic before execution starts

### Requirement: Credential cache identity
r[n8nclone.credentials.cache_identity] The system MUST avoid hashing plaintext secret material into cache keys.

#### Scenario: Cache key uses reference identity
r[n8nclone.credentials.cache_identity.reference]
- GIVEN a node uses a credential during execution
- WHEN the system computes the node cache key
- THEN the key includes the credential reference identity and excludes the resolved secret value

### Requirement: n8n credential placeholders
r[n8nclone.credentials.n8n_json.placeholders] The system MUST import n8n credential references as opaque placeholders and reject plaintext credential material.

#### Scenario: Import credential reference placeholder
r[n8nclone.credentials.n8n_json.placeholders.reference]
- GIVEN an n8n JSON node references a credential identity without plaintext secret material
- WHEN the workflow is imported
- THEN the system stores an opaque credential reference placeholder that must be resolved through a configured provider before execution

#### Scenario: Reject plaintext n8n credential material
r[n8nclone.credentials.n8n_json.placeholders.plaintext]
- GIVEN an n8n JSON import contains plaintext credential values or decrypted credential material
- WHEN import validation runs
- THEN the system rejects the credential data and reports a fatal diagnostic without saving plaintext secrets

#### Scenario: Export keeps credential references opaque
r[n8nclone.credentials.n8n_json.placeholders.export]
- GIVEN a Lattice workflow with credential references is exported to n8n JSON
- WHEN export succeeds for supported nodes
- THEN the system writes only opaque credential reference identities and omits plaintext secret material

### Requirement: Credential provider registry
r[n8nclone.credentials.provider_registry] The system MUST resolve credential references through an explicit configured provider registry.

#### Scenario: Resolve through registered provider
r[n8nclone.credentials.provider_registry.resolve]
- GIVEN a node references a credential provider id and key that exist in the configured registry
- WHEN the node is prepared for execution
- THEN the system resolves the secret through that provider and does not persist the plaintext value

#### Scenario: Unknown provider fails safely
r[n8nclone.credentials.provider_registry.unknown]
- GIVEN a node references a credential provider id that is not configured
- WHEN workflow validation or execution preparation runs
- THEN the system reports a fatal credential diagnostic before invoking the side-effecting node runtime

### Requirement: Provider diagnostics redaction
r[n8nclone.credentials.provider_diagnostics.redaction] The system MUST redact resolved secret material from provider diagnostics and reachability checks.

#### Scenario: Provider test hides secret
r[n8nclone.credentials.provider_diagnostics.redaction.test]
- GIVEN a provider reachability check successfully resolves a secret
- WHEN the result is rendered to a user or persisted
- THEN the system reports success without displaying the secret value

#### Scenario: Provider error hides secret fragments
r[n8nclone.credentials.provider_diagnostics.redaction.error]
- GIVEN a provider failure message contains text derived from secret material
- WHEN the diagnostic is rendered
- THEN the system redacts the secret fragments while preserving safe provider and key context

### Requirement: Environment credential provider
r[n8nclone.credentials.providers.env] The system SHOULD provide an environment-variable credential provider for development and deployment environments.

#### Scenario: Environment secret resolves
r[n8nclone.credentials.providers.env.resolve]
- GIVEN a credential reference targets an environment variable that is present
- WHEN credential resolution runs
- THEN the provider returns the environment value only to the execution boundary

#### Scenario: Missing environment secret fails
r[n8nclone.credentials.providers.env.missing]
- GIVEN a credential reference targets an environment variable that is absent
- WHEN credential resolution runs
- THEN the provider returns a missing-secret error without executing the node runtime

### Requirement: Local encrypted credential vault
r[n8nclone.credentials.providers.local_vault] The system MAY provide a local encrypted vault provider for operator-managed secrets.

#### Scenario: Vault secret resolves with valid key
r[n8nclone.credentials.providers.local_vault.resolve]
- GIVEN a local vault contains an encrypted secret and the configured key source unlocks it
- WHEN credential resolution runs
- THEN the provider returns the plaintext only to the execution boundary and keeps the workflow file free of secret material

#### Scenario: Tampered vault entry is rejected
r[n8nclone.credentials.providers.local_vault.tamper]
- GIVEN a local vault entry fails integrity validation
- WHEN credential resolution runs
- THEN the provider rejects the entry and reports a redaction-safe diagnostic

### Requirement: External command credential provider
r[n8nclone.credentials.providers.command] The system MAY resolve secrets through an explicitly configured external command provider.

#### Scenario: Command provider returns secret
r[n8nclone.credentials.providers.command.resolve]
- GIVEN an external command provider is configured and returns a valid secret response within limits
- WHEN credential resolution runs
- THEN the system supplies the secret to the execution boundary and redacts command output from diagnostics

#### Scenario: Command provider timeout fails safely
r[n8nclone.credentials.providers.command.timeout]
- GIVEN an external command provider exceeds its configured timeout
- WHEN credential resolution runs
- THEN the system fails the node preparation and does not invoke the side-effecting runtime

### Requirement: Provider cache identity
r[n8nclone.credentials.provider_cache_identity] The system MUST include credential provider reference identity but exclude resolved secret values from cache keys.

#### Scenario: Provider-backed cache key avoids plaintext
r[n8nclone.credentials.provider_cache_identity.no_plaintext]
- GIVEN a node resolves a credential through any supported provider
- WHEN the system computes the node cache key
- THEN the key includes provider id, key identity, optional field identity, and excludes the resolved secret bytes

### Requirement: Webhook authentication credentials
r[n8nclone.credentials.webhook_auth] The system MUST resolve webhook authentication material through credential references instead of storing plaintext secrets in trigger definitions.

#### Scenario: Valid webhook secret authenticates request
r[n8nclone.credentials.webhook_auth.valid]
- GIVEN a webhook trigger references a credential provider secret for authentication
- WHEN the provider resolves the secret and the inbound request signature is valid
- THEN the system admits the request without persisting the plaintext secret

#### Scenario: Missing webhook secret rejects request
r[n8nclone.credentials.webhook_auth.missing]
- GIVEN a webhook trigger references a credential provider secret that cannot be resolved
- WHEN an inbound webhook request arrives
- THEN the system rejects the request and does not start the workflow

#### Scenario: Webhook diagnostics redact secret material
r[n8nclone.credentials.webhook_auth.redaction]
- GIVEN webhook authentication fails after resolving secret material
- WHEN the diagnostic is rendered or persisted
- THEN the system omits plaintext secret material and includes only safe trigger and credential reference context

### Requirement: Webhook cookbook secret safety
r[n8nclone.credentials.webhook_cookbook_secret_safety] The system MUST keep webhook cookbook authentication examples credential-reference based and must not expose plaintext secret material in cookbook artifacts or smoke evidence.

#### Scenario: Cookbook uses credential references
r[n8nclone.credentials.webhook_cookbook_secret_safety.references]
- GIVEN the webhook cookbook demonstrates authenticated webhook requests
- WHEN the workflow, docs, and tests are inspected
- THEN they show credential references and setup placeholders instead of plaintext webhook secret values

#### Scenario: Cookbook redaction assertions
r[n8nclone.credentials.webhook_cookbook_secret_safety.redaction]
- GIVEN cookbook smoke tests exercise accepted and rejected authenticated webhook requests
- WHEN responses, diagnostics, run history, logs, and documentation snippets are checked
- THEN authorization headers, bearer values, signature values, raw sensitive payload fields, and credential provider values are absent

### Requirement: n8n parity secret safety
r[n8nclone.credentials.n8n_parity_secret_safety] The system MUST keep supported n8n parity fixtures, checked-in reports, rendered diagnostics, and test output free of plaintext credential material; one designated unsupported rejection fixture MAY contain one bounded plaintext sentinel solely as import-validation input.

#### Scenario: Credential placeholders stay opaque
r[n8nclone.credentials.n8n_parity_secret_safety.placeholders]
- GIVEN a supported n8n parity fixture contains credential references or placeholders
- WHEN the fixture imports and the mapping report is generated
- THEN the report preserves opaque credential identity without exposing plaintext credential values

#### Scenario: Plaintext credential fixture is rejected safely
r[n8nclone.credentials.n8n_parity_secret_safety.plaintext_rejected]
- GIVEN an unsupported n8n parity fixture contains plaintext credential material
- WHEN import validation runs
- THEN the fixture is rejected and diagnostics omit the plaintext value while explaining how to use credential references

### Requirement: Bounded command credential provider
r[lattice.credentials.bounded_command_provider] Lattice MUST execute command credential providers through the shared bounded process mechanism with opaque bounded stdin, a positive deadline, independent stdout and stderr limits, owned teardown, truncation rejection, and diagnostics that omit secret values.

#### Scenario: Command provider resolves inside bounds
r[lattice.credentials.bounded_command_provider.scenario.success]
- GIVEN a configured command credential provider accepts the opaque reference and returns a complete valid response within every process bound
- WHEN Lattice resolves that reference for an admitted effect
- THEN it MUST parse the bounded response and expose the secret only to the existing execution boundary

#### Scenario: Command provider failure remains secret safe
r[lattice.credentials.bounded_command_provider.scenario.failure]
- GIVEN a command credential provider times out, is cancelled, exceeds either stream bound, exits unsuccessfully, or returns malformed output
- WHEN Lattice handles the bounded observation
- THEN it MUST reject credential resolution before the dependent effect and MUST NOT include reference input, stdout, stderr, or resolved secret material in diagnostics or persisted evidence

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
