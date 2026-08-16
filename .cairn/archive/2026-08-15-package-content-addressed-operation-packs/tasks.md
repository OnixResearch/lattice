## Phase 1: Dependencies, references, and baseline

- [x] [depends:introduce-typed-operation-contracts] P1 Begin pack runtime work only after stable operation type, instance, method, output-contract, invocation, and promotion identities exist. r[lattice.operation_packs.manifest]
- [x] [depends:persist-versioned-operation-data] P2 Begin analyzer persistence only after immutable operation resource and file records, exact retrieval, retention, and secret exclusion exist. r[lattice.operation_packs.analyzers]
- [x] [serial] P3 Run and record current runtime registry, Bounded Exec, credential, artifact, Nickel, doctor, Animus manifest, Cairn evidence, local cache, and offline Nix baselines. r[lattice.operation_packs.verification]
- [x] [serial] P4 Record the exact Swamp reference surfaces, AGPL and extension-exception boundary, clean-room adopt/reject decisions, and no source-copy rule. r[lattice.operation_packs.manifest] r[lattice.operation_packs.verification]
- [x] [serial] P5 Inventory current Onix stack build, Artifact authentication, Valence, Cairn, Basalt, UCAN, Radicle, mirror, and release evidence that can satisfy bounded admission profiles. r[lattice.operation_packs.admission]

## Phase 2: Manifest, identity, and path safety

- [x] [serial] M1 Add typed Nickel contracts for pack, operation, runtime, analyzer, skill, dependency, capability, platform, evidence, release, license, bound, and non-claim fields. r[lattice.operation_packs.manifest]
- [x] [serial] M2 Add strict Rust manifest validation with unknown-field, duplicate, scope, path, Unicode normalization, bound, schema-version, and contradiction diagnostics. r[lattice.operation_packs.manifest]
- [x] [serial] M3 Add canonical BLAKE3 manifest, content, dependency-closure, runtime, analyzer, skill, lock, and pack identities while excluding display and mutable registry metadata. r[lattice.operation_packs.identity]
- [x] [serial] M4 Add capability-rooted archive or directory inspection that rejects absolute paths, parent traversal, symlinks, duplicate normalized paths, unsafe file kinds, oversize, and undeclared content. r[lattice.operation_packs.identity]
- [x] [parallel] M5 Add positive and negative manifest, canonicalization, path, collision, changed-content, same-identity conflict, and display-metadata stability fixtures. r[lattice.operation_packs.manifest] r[lattice.operation_packs.identity]

## Phase 3: Admission, locks, and external runtimes

- [x] [serial] A1 Add pure profile-driven admission over measured bytes, dependency closure, platform, license, Mantle, Artifact, Valence, Cairn, Basalt, UCAN, and policy facts. r[lattice.operation_packs.admission]
- [x] [serial] A2 Add exact lock creation and validation for pack, dependencies, platform, runtime, analyzers, skills, trust profile, and source locator metadata. r[lattice.operation_packs.admission]
- [x] [parallel] A3 Add negative missing-required, invalid-optional-present, stale, mismatched, cycle, platform, license, trust, capability, and lock-drift cases. r[lattice.operation_packs.admission]
- [x] [serial] R1 Add reviewed executable runtime descriptors and launch-label protocols through existing Bounded Exec without manifest command strings or in-process code loading. r[lattice.operation_packs.runtime]
- [x] [serial] R2 Reject TypeScript, JavaScript, Deno, Node, shared-library, arbitrary command, undeclared host callback, and unsupported Wasm runtime requests. r[lattice.operation_packs.runtime]
- [x] [parallel] R3 Add runtime positive and negative cases for exact executable identity, argv protocol, environment allowlist, roots, credentials, timeout, cancellation, truncation, teardown, output contract, and unknown outcome. r[lattice.operation_packs.runtime]

## Phase 4: Bounded analyzers

- [x] [serial] N1 Add analyzer manifests for scope, data view, required status, runtime identity, input bounds, output schema, Markdown, retention, sensitivity, and non-claims. r[lattice.operation_packs.analyzers]
- [x] [serial] N2 Add a pure analyzer selection and input-view planner that discloses only declared safe run metadata and exact operation-data references. r[lattice.operation_packs.analyzers]
- [x] [serial] N3 Execute analyzers through admitted external runtimes and validate structured and Markdown outputs before operation-data publication. r[lattice.operation_packs.analyzers]
- [x] [serial] N4 Make missing, failed, timed-out, truncated, or invalid required analyzers fail the parent result while retaining typed visible optional failures. r[lattice.operation_packs.analyzers]
- [x] [parallel] N5 Add negative credential, environment, private path, unrestricted store, undeclared data, oversized Markdown, output mismatch, evidence-role promotion, and authority escalation cases. r[lattice.operation_packs.analyzers]

## Phase 5: Passive skills and routing evaluation

- [x] [serial] S1 Add pack skill metadata, body, reference, host, trust, and BLAKE3 identity validation through existing Animus-compatible passive-resource contracts where exact dependency evidence permits. r[lattice.operation_packs.skills]
- [x] [serial] S2 Add a product-owned Lattice gateway skill for operation discovery, instance authoring, direct run, promotion, workflows, data inspection, pack diagnosis, and safety boundaries. r[lattice.operation_packs.skills]
- [x] [serial] S3 Add explicit host installation and removal adapters that track exact installed skill identities and never execute skill content. r[lattice.operation_packs.skills]
- [x] [parallel] S4 Add positive routing cases and negative neighboring cases for Cairn lifecycle, OnixOS configuration, Mantle builds, Valence evidence, generic shell tasks, and unavailable operation types. r[lattice.operation_packs.skills]
- [x] [parallel] S5 Run structural routing tests deterministically and the existing manifest-driven multi-model evaluation lane when configured, recording skipped live-provider evaluation without inventing success. r[lattice.operation_packs.skills]

## Phase 6: Registry and doctor

- [x] [serial] G1 Add local and explicitly configured mirror registry catalog validation with exact pack identities, safe descriptions, platforms, channels, deprecation, supersession, and content locators. r[lattice.operation_packs.registry]
- [x] [serial] G2 Add explicit search, info, exact selection, freshness, and restore plans that never auto-install, auto-update, change locks, or contact a registry during ordinary execution. r[lattice.operation_packs.registry]
- [x] [serial] G3 Add metadata-only beta-to-rc-to-stable promotion that preserves exact pack bytes and identity and rejects reverse, ambiguous, missing, or rebuilt promotion. r[lattice.operation_packs.registry]
- [x] [parallel] G4 Add offline, mirror mismatch, duplicate candidate, deprecated, superseded, unavailable, stale cache, passive-network, and auto-update denial fixtures. r[lattice.operation_packs.registry]
- [x] [serial] D1 Extend doctor with manifest, identity, lock, dependencies, platform, runtime, capabilities, evidence, analyzers, skills, cache, and registry facts. r[lattice.operation_packs.doctor]
- [x] [serial] D2 Add bounded no-effect probe contracts and reject ordinary effectful methods, production credentials, mutation, install, update, and arbitrary probe output. r[lattice.operation_packs.doctor]
- [x] [parallel] D3 Add deterministic healthy, warning, fatal, skipped, redacted, and end-to-end drift fixtures with actionable remediation. r[lattice.operation_packs.doctor]

## Phase 7: Dogfood and closeout

- [x] [serial] F1 Package the read-only Cairn operation, one bounded summary analyzer, and gateway skill references as a local exact fixture pack. r[lattice.operation_packs.verification]
- [x] [serial] F2 Prove offline lock restore, metadata discovery without loading, read-only execution, typed data publication, analyzer behavior, skill disclosure, registry inspection, and doctor health through production entrypoints. r[lattice.operation_packs.verification]
- [x] [parallel] F3 Add end-to-end tamper, dependency, trust, path, license, runtime, analyzer, skill, registry, credential, capability, leak, and overclaim fixture packs. r[lattice.operation_packs.verification]
- [x] [serial] F4 Update README and operator docs with manifest authoring, identity, locks, admission profiles, runtime limits, analyzers, skills, registry, promotion, deprecation, doctor, offline use, license boundary, and non-claims. r[lattice.operation_packs.manifest] r[lattice.operation_packs.registry]
- [x] [serial] F5 Run focused tests, formatting, Clippy, `scripts/check.rs`, Octet, Cairn validation and gates, supply-chain fixture rails, and bounded Nix validation. r[lattice.operation_packs.verification]
- [x] [serial] F6 Review dry-run sync and archive plans, make sure that every requirement reaches operation-packs, and archive only after current positive and negative evidence passes. r[lattice.operation_packs.verification]
