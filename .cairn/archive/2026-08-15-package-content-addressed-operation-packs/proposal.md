## Why

Typed operations become broadly useful when teams can package, inspect, pin, share, deprecate, and promote them without granting ambient code-loading authority. Lattice also needs a bounded home for post-run analyzers and product-specific agent guidance.

Swamp demonstrates useful extension-product mechanics: scoped manifests, explicit trust, immutable release bytes, metadata-only promotion, deprecation, discoverable content, bundled passive skills, and post-run reports. Lattice will adapt these mechanics through typed Nickel, BLAKE3, exact runtime identities, Bounded Exec or separately admitted Wasm components, Mantle build evidence, Artifact authentication, Valence linkage, Cairn promotion gates, and Basalt/UCAN authority.

This is a clean-room design. It does not copy Swamp source or use its extension exception as permission for reimplementation.

## What Changes

- Add a typed Nickel operation-pack manifest for operation types, runtime artifacts, analyzers, passive skills, dependencies, capabilities, platforms, release metadata, and non-claims. r[lattice.operation_packs.manifest]
- Compute one canonical BLAKE3 pack identity over normalized manifest and exact content identities while keeping display version and channel outside authority. r[lattice.operation_packs.identity]
- Admit packs only through explicit trust, exact lock entries, compatible licenses, safe paths, verified bytes, dependency closure, and current policy facts. r[lattice.operation_packs.admission]
- Execute pack operations only through reviewed external runtime boundaries. Lattice will not load arbitrary TypeScript, JavaScript, native libraries, or manifest code in-process. r[lattice.operation_packs.runtime]
- Add bounded post-run analyzers that receive declared data views, emit typed Markdown and structured results, and can be optional or required. r[lattice.operation_packs.analyzers]
- Add a passive Lattice gateway skill with progressive disclosure and positive and negative routing evaluation. r[lattice.operation_packs.skills]
- Add explicit local or mirrored registry catalogs with immutable release bytes, beta/rc/stable promotion labels, deprecation, supersession, exact pins, and no automatic install or update. r[lattice.operation_packs.registry]
- Extend doctor with manifest, lock, dependency, runtime, policy, analyzer, and skill checks plus bounded no-effect probes. r[lattice.operation_packs.doctor]
- Add end-to-end positive and negative supply-chain, execution, analyzer, skill, registry, redaction, and claim-boundary evidence. r[lattice.operation_packs.verification]

## Success Criteria

- Pack inspection does not load executors, resolve credentials, contact registries, or execute analyzer or skill content.
- Exact pack identity changes when any authoritative manifest field or content identity changes.
- Promotion from beta to rc or stable changes catalog metadata only. Pack bytes and BLAKE3 identity remain unchanged.
- Untrusted, stale, tampered, path-escaping, dependency-incomplete, license-incompatible, capability-widened, or runtime-mismatched packs fail before installation or dispatch.
- Required analyzer failure makes the parent run non-successful. Optional analyzer failure remains visible and cannot become success evidence.
- Skills remain passive text. Routing evaluation includes positive and neighboring negative cases.
- Registry freshness is explicit and read-only. Normal commands do not auto-fetch, auto-update, or emit passive network nags.
- The first dogfood pack bundles the read-only Cairn operation type without mutable lifecycle authority.

## False Completion

A tar archive with a manifest and no exact content, trust, capability, runtime, or dependency binding is not a pack.

A CalVer label, registry row, download URL, process exit, signature alone, or passing analyzer does not establish pack authority or correctness.

An in-process dynamic extension loader, arbitrary report code with ambient store access, or skill text that grants runtime capabilities is false completion.

## Impact

- **Lattice core**: pure manifest, identity, dependency, trust, channel, analyzer-selection, skill-disclosure, registry-selection, and doctor-fact validation.
- **Lattice shell**: pack loading from capability roots, exact byte measurement, lock persistence, explicit local or mirrored fetch, runtime dispatch, analyzer execution, skill installation adapters, and doctor probes.
- **Mantle**: optional authoritative builder for reviewed executables or Wasm components and associated build evidence. Lattice remeasures consumer bytes.
- **Artifact, Valence, Cairn, Basalt, and UCAN**: supply bounded authentication, identity linkage, promotion evidence, and authority facts without taking Lattice runtime ownership.
- **Animus**: supplies existing authority-free plugin, tool, and skill manifest DTO patterns. It does not load or execute packs.
- **Testing**: local fixture packs, tampering, trust, dependency, path, license, runtime, analyzer, routing, registry, offline, and non-claim cases.

## Out of Scope

- A public hosted marketplace, payment system, popularity ranking, telemetry leaderboard, remote code review service, or automatic publisher trust.
- In-process TypeScript, JavaScript, Deno, Node, native-library, or arbitrary plugin loading.
- Passive automatic updates, background registry traffic, semantic-version range authority, or CalVer as content identity.
- Claims for maintainer trust, build correctness, analyzer correctness, skill routing correctness, sandbox completeness, production readiness, or release eligibility.

## Dependencies

- Local Cairn change: `introduce-typed-operation-contracts`.
- Local Cairn change: `persist-versioned-operation-data`.

## Affected Specs

- `operation-packs`: manifest, identity, admission, runtime, analyzers, skills, registry, doctor, and verification.
