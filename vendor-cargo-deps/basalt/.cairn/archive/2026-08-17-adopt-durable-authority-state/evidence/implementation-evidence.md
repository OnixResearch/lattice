# Durable Authority State second-consumer evidence

## Baseline

Before the composition change, all 22 compact UCAN enforcement integration tests passed.
The accepted protocol-profile change already pinned vendored UCAN revision `c483c7b58c42ec6636e9b8b8c0f73a2b609bf21e`.

## Source

- Canonical RID: `rad:z3kGDT73ja8rewvGLwJFLF5MTx1ih`
- Admitted adapter: `https://seed.radicle.garden/z3kGDT73ja8rewvGLwJFLF5MTx1ih.git`
- Reviewed object: `a0de79303f3f45c18a34fff535b690a85cb3cf59`
- Direct packages: `replay-ledger-core` and `revocation-view-core`

Cargo declares both packages directly at the same object.
The lockfile contains the same source identity twice.
Rust and Nix source checks reject GitHub, sibling, branch, tag, and path fallback forms.

## Composition

`BasaltDurableAuthority` composes UCAN's published durable replay and revocation adapters.
It adds no replay or revocation transition logic.
The type remains generic over caller-owned `ReplayLedgerPort` and `RevocationViewPort` implementations.

`DurableAuthorityVerificationInputs` carries caller-owned keys, proofs, caveat policy, explicit verification time, and limits.
The composition returns Basalt's existing `UcanEnforcementBoundaries`.
Basalt policy and compact-profile checks still run before token verification and durable replay.
UCAN still calls replay once as its final state-consuming boundary.

## Pilot tests

The first end-to-end compact invocation reserved one shared replay slot and returned live Basalt authority.
The repeated invocation denied as replay and did not perform a second write.

Current, complete, authenticated revocation evidence allowed lookup.
Partial, unauthenticated, missing, unavailable, and failed view cases denied before any replay write.

## Validation

- Exact-source positive and negative tests passed: 2 of 2.
- End-to-end durable composition tests passed: 2 of 2.
- Existing compact enforcement tests remained green.
- Formatting and Clippy with denied warnings passed.
- The direct release verifier passed 188 default-feature tests and 201 CLI-feature tests.
- Property, Steel, authority, semantics, token-flow, policy, identity, source, package, and documentation checks passed.
- Cairn proposal, design, task, and validation gates passed.
- Cairn traceability reported 16 of 224 accepted identifiers referenced.
- All 7 new durable-authority identifiers are referenced; 208 inherited identifiers remain missing.
- `nix flake check path:$PWD -L` passed with local builders.

## Bounded portfolio search

The exact goal was a second real consumer without copied state logic or transferred authority.
False completion included test-only source declarations, a concrete database inside Basalt, copied replay transitions, state evidence treated as authorization, or replay before Basalt policy.

Three correlated serial lenses were used because subagents were not authorized:

| Family | Mechanism | Evidence | State |
|---|---|---|---|
| UCAN-adapter-composition | Basalt selects UCAN's published adapters over generic shared ports | end-to-end positive and negative tests pass | validated |
| copied-Basalt-adapter | duplicate UCAN-to-core mapping in Basalt | rejected because the published adapter already owns that mapping | falsified |
| concrete-Redb-shell | Basalt selects and owns one database | rejected because applications retain persistence and outage authority | falsified |

The surviving composition was audited against duplicate replay, partial and unauthenticated snapshots, missing and failed views, fallback sources, and replay-order regressions.
The declared budget was three lenses, one exact source, two generic ports, and one full release gate.

## Non-claims

This pilot does not prove global replay exclusion, revocation authenticity, backend honesty, effect completion, indefinite source availability, final UCAN `1.0.0` compatibility, release eligibility, or whole-system authorization correctness.
Basalt retains application policy and action mapping.
UCAN retains protocol authority.
Callers retain persistence, time, authenticity, recovery, outage, and effect authority.
