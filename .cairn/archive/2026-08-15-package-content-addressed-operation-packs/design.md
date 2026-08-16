## Context

The operation-contract change provides typed reusable methods, data-only instances, exact invocation plans, and one read-only Cairn pilot. The operation-data change provides immutable resources and files, exact versions, bounded query, retention, and selected evidence projection.

The remaining distribution problem has several separate trust questions:

- What content belongs to one operation pack?
- Which exact bytes and contracts are selected?
- Who built or authenticated them?
- Which capabilities can Lattice expose at runtime?
- Which analyzer and skill content can receive context?
- Which registry metadata is advisory, and which identity is authoritative?

Swamp provides a useful product reference for manifests, extension trust, channels, reports, skills, deprecation, and explicit freshness. Onix stack components provide stronger bounded mechanisms for content identity, build evidence, authority, and lifecycle gates.

## Success Contract

Completion requires one typed pack manifest, canonical content identity, explicit lock and trust policy, safe archive/path handling, external runtime boundary, bounded analyzers, passive skills, explicit registry selection, doctor diagnostics, and positive and negative fixture packs.

A manifest parser without production installation and execution evidence is incomplete. A registry UI without exact pack identity and trust denial is incomplete.

## Decisions

### Decision: Nickel owns the pack manifest

**Choice:** `manifest.ncl` is the human-authored source. It exports a strict runtime manifest that Rust revalidates.

The manifest includes:

- schema version, scoped pack ID, description, safe labels, and display version;
- exact operation type and type-revision identities;
- runtime artifact roles, platforms, media classes, byte identities, launch labels, and capability classes;
- analyzer definitions and declared data-view contracts;
- passive skill metadata, body and reference identities, and supported agent hosts;
- dependency pack IDs and exact BLAKE3 identities;
- required build, authentication, evidence, lifecycle, and authority profiles;
- release channel eligibility, deprecation, and supersession metadata;
- license identifiers and review references;
- named bounds and required non-claims.

Paths are safe relative locators and never enter pack authority without measured content identities.

**Rationale:** Nickel provides typed review and generated runtime data without making a second executable manifest language.

### Decision: BLAKE3 content identity is authoritative

**Choice:** A pack identity binds normalized authoritative manifest fields and exact content identities. It excludes display version, channel, publication time, mirror URL, local path, popularity, and mutable registry presentation.

A lock entry binds pack identity, manifest identity, dependency closure, platform selection, runtime artifact identities, analyzer identities, skill identities, trust profile, and source locator metadata.

Any same-identity byte conflict fails closed. Equivalent bytes under different mirror paths remain the same candidate.

**Rationale:** Release labels help humans. Exact content identity supports safe execution and replay.

### Decision: admission composes bounded stack evidence

**Choice:** Pack admission consumes a profile-selected set of already loaded facts:

- measured manifest and content BLAKE3 identities;
- safe archive and path validation;
- dependency closure and platform compatibility;
- license compatibility decision;
- optional Mantle build bundle and exact output identity;
- optional Artifact authentication result;
- optional Valence `RecordedOnly` linkage;
- current Cairn promotion receipt or release evidence;
- Basalt/UCAN install and run authority facts when policy requires them.

The pure admission core validates agreement. It does not run Mantle, verify tokens, fetch keys, inspect registries, read archives, or grant trust from missing evidence.

Profiles state which facts are required. Optional absent evidence is skipped only when policy permits absence. Optional present evidence must still validate.

**Rationale:** No one stack layer proves the complete pack-admission claim.

### Decision: no arbitrary in-process code loading

**Choice:** The first runtime class uses reviewed executable artifacts through existing Bounded Exec. The manifest supplies an admitted launch label and argument protocol, not an arbitrary command string.

A later Wasm component class can use Mantle-built, Octet-validated portable components only after a separate Lattice runtime profile proves WIT/world, capability, resource, timeout, memory, fuel, and host-function boundaries.

Lattice does not load pack TypeScript, JavaScript, Deno modules, Node packages, shared libraries, or manifest-provided host callbacks in-process.

**Rationale:** Dynamic in-process extensions would bypass current runtime, credential, process, and capability boundaries.

### Decision: analyzers are bounded operation components

**Choice:** An analyzer declares scope (`method`, `operation`, or `workflow`), input data-view contract, required or optional status, runtime artifact identity, bounds, output schema, Markdown allowance, retention, sensitivity, and non-claims.

After execution, Lattice builds a bounded analyzer input containing only selected safe run metadata and exact operation-data references. The analyzer receives no ambient repository, store, credential, environment, network, or workflow authority.

Analyzer outputs are validated before they become operation-data records. A required analyzer that is missing, fails, times out, truncates, or violates output policy makes the parent result non-successful. An optional failure remains a visible typed outcome.

An analyzer cannot change the original operation observation, authorize another effect, or promote evidence role.

**Rationale:** Reports are useful only when their failure and authority semantics are explicit.

### Decision: skills remain passive and progressively disclosed

**Choice:** Packs can include passive skill resources. Metadata is available for routing only after pack trust and safe description validation. The body loads only after one explicit trusted selection with matching body identity. References load through bounded on-demand requests.

The Lattice product supplies one gateway skill that teaches agents to inspect operation types, create instances, run one-off methods, promote invocations, author workflows, inspect operation data, and diagnose packs.

Trigger evaluation uses positive cases and negative neighboring cases. It tests routing only and cannot establish task success or grant operation authority.

Animus skill-manifest contracts can supply reusable validation shapes through an exact dependency. Lattice owns product text, installation adapters, host directories, evaluation execution, and operator policy.

**Rationale:** Agent guidance belongs near the product while generic passive-resource contracts remain reusable.

### Decision: registry catalogs are explicit metadata layers

**Choice:** The first registry supports local files and explicitly configured mirrors. A signed or authenticated remote transport can be added behind a separate adapter. Registry catalogs list exact pack identities, channels, deprecation, supersession, platforms, safe descriptions, and content locators.

Beta, rc, and stable promotion changes registry metadata only. The selected pack bytes and BLAKE3 identity do not change. Stable is never inferred from age, version text, or popularity.

Search and freshness commands are explicit. Normal operation uses the lock and local cache. It does not contact a registry, auto-install, auto-update, or display passive network nags.

Deprecation is advisory unless policy blocks it. Supersession never rewrites locks automatically.

**Rationale:** Registry convenience must not weaken reproducibility or offline use.

### Decision: doctor checks the complete integration chain

**Choice:** Doctor gathers bounded facts for manifest validation, pack identity, lock agreement, dependency closure, platform selection, runtime binding, capability policy, required evidence, analyzer availability, skill identity, cache presence, and registry configuration.

Optional no-effect probes use pack-declared synthetic inputs and a dedicated probe method or executor self-description. Doctor never invokes an ordinary effectful method, resolves production credentials, mutates workflow state, installs updates, or trusts arbitrary probe output.

Stable diagnostics include remediation and safe identities only.

**Rationale:** Integration drift often appears as silent missing discovery or runtime mismatch.

### Decision: dogfood with one local Cairn pack

**Choice:** Package the read-only Cairn operation type, its safe metadata, optional summary analyzer, and Lattice gateway references as a local fixture pack.

The fixture has no mutation method, registry network requirement, production credential, or authority escalation. It exercises manifest, lock, runtime, analyzer, skill, doctor, and offline restoration paths.

**Rationale:** A real bounded pack is stronger evidence than manifest-only unit tests.

## Pack Flow

```text
manifest.ncl + exact content
  -> Rust manifest and path validation
  -> BLAKE3 pack identity and dependency closure
  -> profile-selected build/auth/evidence/authority fact validation
  -> explicit lock and local materialization
  -> metadata discovery without executor load
  -> ordinary Lattice invocation admission
  -> external bounded runtime
  -> typed operation data
  -> optional or required bounded analyzers
```

## Validation Strategy

Positive fixtures cover local pack creation, exact lock restore, offline discovery, Cairn method execution, required and optional analyzers, gateway skill metadata and body disclosure, explicit registry search, metadata-only channel promotion, deprecation, supersession, and doctor health.

Negative fixtures cover traversal, symlinks, duplicate paths, Unicode collisions, wrong hash, same identity with different bytes, dependency cycle, missing dependency, platform mismatch, license denial, stale build evidence, authentication mismatch, capability widening, in-process loader request, arbitrary command, analyzer leak, required analyzer failure, skill body mismatch, routing collision, registry ambiguity, passive network access, and auto-update.

## Adversarial Audit

The final audit must try to show that:

- listing a pack loads or executes code;
- a display version or channel substitutes for BLAKE3 identity;
- a manifest path escapes its capability root;
- optional evidence bypasses required policy;
- an analyzer receives credentials or unrestricted store access;
- a skill description activates an untrusted body;
- stable promotion rebuilds or changes bytes;
- registry failure changes an exact locked execution;
- doctor invokes an ordinary effectful method;
- a successful pack is described as correct, trusted, or release-ready without matching evidence.

Any successful counterexample blocks closeout.

## Risks and Trade-offs

- Multi-layer evidence can make admission complex. Profiles and deterministic issue codes keep required facts visible.
- External executables have process isolation but not sandbox proof. Capability and executable authorization remain separate requirements.
- A public registry creates governance and availability obligations. The first slice remains local or mirror-based.
- Analyzer failures can surprise operators. Required status is explicit in reviewed instance or workflow policy.
- Multi-model skill routing evaluation costs time and provider access. Deterministic structural tests remain mandatory when live model evaluation is unavailable.

## Non-Claims

This design does not prove pack maintainer trust, source correctness, build correctness, authentication authority beyond supplied facts, analyzer correctness, skill routing correctness, model correctness, sandbox completeness, remote registry availability, production readiness, or release eligibility.
