# Operation Packs Specification

## Purpose

Defines the `operation-packs` capability.

## Requirements

### Requirement: Operation packs use one typed manifest

r[lattice.operation_packs.manifest] Lattice MUST use typed Nickel as the authored source for operation-pack manifests and MUST revalidate generated manifest data in Rust before inspection, locking, admission, installation, or execution.

#### Scenario: Complete manifest is admitted

r[lattice.operation_packs.manifest.valid]
- GIVEN a manifest declares one scoped pack ID, schema version, exact operation types, runtime artifacts, analyzers, passive skills, dependencies, capabilities, platforms, evidence profiles, release metadata, licenses, bounds, and non-claims
- WHEN Rust manifest validation runs
- THEN Lattice MUST return deterministic safe content metadata and exact declared content roles
- AND inspection MUST NOT load executors, resolve credentials, contact registries, or execute analyzers or skill content.

#### Scenario: Manifest is malformed or contradictory

r[lattice.operation_packs.manifest.invalid]
- GIVEN a manifest has unknown fields, duplicate normalized entries, unsafe paths, unsupported versions, missing bounds, capability contradictions, dependency ambiguity, missing licenses, or missing non-claims
- WHEN validation runs
- THEN Lattice MUST reject the pack before lock, cache, install, runtime, analyzer, skill, or registry mutation
- AND it MUST NOT infer omitted trust, capability, or runtime facts.

### Requirement: Pack authority uses canonical content identity

r[lattice.operation_packs.identity] Lattice MUST compute a domain-separated BLAKE3 pack identity over normalized authoritative manifest fields and exact content identities while excluding display version, channel, publication time, mirror URL, local path, and popularity metadata.

#### Scenario: Same content under another locator is identical

r[lattice.operation_packs.identity.stable]
- GIVEN two candidates contain equivalent normalized authoritative manifest data and exact content bytes but use different safe mirror locators or display versions
- WHEN pack identity is computed
- THEN their authoritative pack identities MUST be equal
- AND the differing display or locator metadata MUST remain reviewable outside execution authority.

#### Scenario: Authoritative content changes

r[lattice.operation_packs.identity.changed]
- GIVEN an operation contract, runtime artifact, analyzer, skill body, dependency identity, capability, platform, required evidence profile, bound, license, or non-claim changes
- WHEN pack identity is recomputed
- THEN the BLAKE3 pack identity MUST change or validation MUST fail closed
- AND same-identity different-byte content MUST be rejected as a conflict.

### Requirement: Pack admission requires exact policy-selected evidence

r[lattice.operation_packs.admission] Lattice MUST admit a pack only when measured bytes, safe paths, dependency closure, platform, license, lock, trust policy, capability policy, and every profile-required build, authentication, evidence, lifecycle, and authority fact agree.

#### Scenario: Exact pack satisfies its admission profile

r[lattice.operation_packs.admission.valid]
- GIVEN measured manifest and content identities, dependency closure, platform, license decision, lock, runtime facts, and required Mantle, Artifact, Valence, Cairn, Basalt, or UCAN evidence match the selected policy profile
- WHEN pure pack admission runs
- THEN Lattice MUST return one exact admitted pack and lock plan
- AND each evidence layer MUST retain its bounded claim and non-claims.

#### Scenario: Required or present evidence is invalid

r[lattice.operation_packs.admission.denied]
- GIVEN required evidence is absent or any supplied optional evidence is stale, malformed, mismatched, overclaimed, unauthorized, license-incompatible, dependency-incomplete, platform-incompatible, or capability-widened
- WHEN admission runs
- THEN Lattice MUST deny with deterministic diagnostics before installation or execution
- AND it MUST NOT downgrade the profile, omit present invalid evidence, choose another dependency, or trust registry metadata alone.

### Requirement: Pack runtimes remain external and bounded

r[lattice.operation_packs.runtime] Lattice MUST execute admitted operation-pack methods through reviewed external runtime boundaries and MUST NOT load arbitrary pack code in-process.

#### Scenario: Reviewed executable method runs

r[lattice.operation_packs.runtime.executable]
- GIVEN an admitted pack binds one exact executable artifact, launch-label protocol, argv shape, environment allowlist, capability policy, credential bindings, roots, bounds, and output contract
- WHEN method dispatch runs
- THEN Lattice MUST use the existing bounded process shell and validate the result before downstream use
- AND the manifest MUST NOT provide an arbitrary command string or undeclared host callback.

#### Scenario: Unsupported dynamic loader is requested

r[lattice.operation_packs.runtime.denied]
- GIVEN a pack requests in-process TypeScript, JavaScript, Deno, Node, shared-library, arbitrary command, undeclared callback, or unsupported Wasm execution
- WHEN runtime admission runs
- THEN Lattice MUST reject before code loading or process dispatch
- AND it MUST NOT fall back to a shell node, interpreter, another runtime, or ambient host loader.

### Requirement: Post-run analyzers are bounded and policy-visible

r[lattice.operation_packs.analyzers] Lattice MUST run pack analyzers only with declared safe data views, exact runtime identities, finite bounds, typed outputs, explicit required or optional status, retention, sensitivity, and non-claims.

#### Scenario: Required analyzer succeeds

r[lattice.operation_packs.analyzers.valid]
- GIVEN a required analyzer receives only its admitted method, operation, or workflow metadata and exact safe operation-data references
- WHEN its external runtime completes within bounds and outputs contract-valid structured data and Markdown
- THEN Lattice MUST persist admitted analyzer outputs as operation data and retain analyzer identity and scope
- AND analyzer output MUST NOT replace or strengthen the original operation observation.

#### Scenario: Analyzer fails or asks for ambient authority

r[lattice.operation_packs.analyzers.failure]
- GIVEN an analyzer is missing, times out, truncates, returns invalid output, requests credentials, environment, private paths, unrestricted store access, undeclared data, mutation, or evidence-role promotion
- WHEN analyzer admission or execution runs
- THEN Lattice MUST reject the analyzer output and make a required analyzer failure non-successful for the parent result
- AND an optional failure MUST remain a visible typed outcome without becoming success evidence.

### Requirement: Pack skills are passive and progressively disclosed

r[lattice.operation_packs.skills] Lattice MUST treat operation-pack skills as passive identity-bound text resources, disclose bodies and references only after trusted selection, and test routing with positive and negative neighboring cases.

#### Scenario: Trusted gateway skill loads on demand

r[lattice.operation_packs.skills.valid]
- GIVEN the Lattice gateway skill metadata is trusted, its body identity matches, and an operation task matches the reviewed routing description
- WHEN the host selects and loads the skill
- THEN it MUST disclose the bounded body and only requested matching references
- AND the skill MUST grant no operation, credential, workflow, pack, registry, or host authority.

#### Scenario: Skill is untrusted, mismatched, or misrouted

r[lattice.operation_packs.skills.denied]
- GIVEN skill metadata or body is untrusted, duplicated, stale, identity-mismatched, unsafe, or a query belongs to Cairn, OnixOS, Mantle, Valence, generic shell work, or an unavailable operation
- WHEN skill validation or routing evaluation runs
- THEN the body MUST remain outside model context or the negative routing case MUST pass
- AND a routing result MUST NOT be reported as task correctness or execution authorization.

### Requirement: Registry channels never replace exact pack identity

r[lattice.operation_packs.registry] Lattice MUST use explicit local or configured mirror catalogs for pack discovery and MUST keep exact pack identities and lock entries authoritative over channels, display versions, deprecation, supersession, and freshness metadata.

#### Scenario: Existing bytes are promoted between channels

r[lattice.operation_packs.registry.promote]
- GIVEN one exact pack identity and bytes are cataloged in beta or rc and policy admits forward promotion
- WHEN promotion to rc or stable runs
- THEN only catalog metadata MUST change and the exact pack identity and bytes MUST remain unchanged
- AND reverse, ambiguous, missing, or rebuilt promotion MUST fail closed.

#### Scenario: Ordinary execution is offline

r[lattice.operation_packs.registry.offline]
- GIVEN a workflow uses one exact valid lock and locally available pack bytes
- WHEN ordinary inspection or execution runs while the registry is unavailable
- THEN Lattice MUST use the exact local locked candidate without registry access
- AND it MUST not auto-fetch, auto-update, rewrite locks, select a successor, or emit passive network update warnings.

#### Scenario: Deprecation or freshness is inspected

r[lattice.operation_packs.registry.advisory]
- GIVEN an operator explicitly requests registry info, freshness, deprecation, or supersession status
- WHEN the configured catalog is available or unavailable
- THEN Lattice MUST report bounded advisory metadata and distinguish unavailable knowledge from current status
- AND advisory metadata MUST not mutate installed content or prove pack safety.

### Requirement: Doctor validates pack integration without ordinary effects

r[lattice.operation_packs.doctor] Lattice MUST collect deterministic doctor facts for pack manifests, identities, locks, dependencies, platforms, runtimes, capabilities, evidence, analyzers, skills, cache, and registry configuration without invoking ordinary effectful methods.

#### Scenario: Pack integration is healthy

r[lattice.operation_packs.doctor.valid]
- GIVEN one installed pack has current manifest, lock, dependencies, runtime, policy, evidence, analyzer, skill, cache, and optional no-effect probe facts
- WHEN doctor runs
- THEN Lattice MUST report stable healthy facts and safe identities without resolving production credentials or mutating workflow state
- AND the result MUST not claim operation correctness or external-system health beyond the exact probe.

#### Scenario: Probe or integration drifts

r[lattice.operation_packs.doctor.invalid]
- GIVEN a manifest, lock, dependency, runtime, capability, evidence, analyzer, skill, cache, registry, or no-effect probe fact is missing, stale, mismatched, unsafe, or unavailable
- WHEN doctor runs
- THEN Lattice MUST report deterministic warning or fatal diagnostics with bounded remediation
- AND it MUST not install, update, execute an ordinary method, use production credentials, or trust arbitrary probe text.

### Requirement: Operation packs have positive and negative product evidence

r[lattice.operation_packs.verification] Lattice MUST validate operation packs through positive and negative production manifest, identity, admission, lock, runtime, analyzer, skill, registry, doctor, offline, redaction, and claim-boundary entrypoints.

#### Scenario: Read-only Cairn pack passes end to end

r[lattice.operation_packs.verification.positive]
- GIVEN the reviewed local Cairn fixture pack has exact content, dependencies, lock, runtime, analyzer, skill, registry, and policy facts
- WHEN the focused pack rail runs offline
- THEN production entrypoints MUST inspect without loading, execute the read-only method, persist typed data, run declared analyzers, disclose the trusted skill, and report doctor health
- AND no mutation method, production credential, remote registry, or unsupported authority MAY be present.

#### Scenario: Adversarial pack fails before prohibited behavior

r[lattice.operation_packs.verification.negative]
- GIVEN a fixture attempts tampering, path escape, identity conflict, dependency confusion, license bypass, trust downgrade, capability widening, dynamic loading, analyzer leakage, skill injection, registry ambiguity, auto-update, credential exposure, or overclaiming
- WHEN the focused pack rail runs
- THEN the attempt MUST fail before prohibited installation, loading, dispatch, disclosure, persistence, mutation, or claim
- AND passing pack evidence MUST NOT establish maintainer trust, implementation correctness, sandbox completeness, production readiness, or release eligibility.
