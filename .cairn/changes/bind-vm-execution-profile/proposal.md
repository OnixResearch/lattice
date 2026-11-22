# Bind implementation attempts to a VM execution profile and a host model gateway

Tracking: [Lattice #10](https://github.com/OnixResearch/lattice/issues/10). Slice S1 of the relocation roadmap. Lattice #11 and #12 consume this change's C1 and C2 identities; Kiln #51, #52 and #54 consume all four records as fixtures.

## Why

Kiln's M1 guest runs headless Lattice workflows inside a no-NIC Cloud Hypervisor guest. The #6 implementation attempt does not run there yet. Its model broker resolves a credential and calls the upstream directly, which a guest must never do. A guest attempt also needs records that name exactly what it runs under, so that a crossed image, environment, envelope, attempt, workspace or mode fails before any effect. Its deadline must be enforced by the host, not by a clock the guest controls. Its model budget must survive a move between hosts.

## What Changes

- Add `lattice.vm-execution-profile.v1` (C1): a Nickel-authored, JSON-admitted profile covering architecture, VMM, guest manifest schema, a guest network of `none` only, the gateway vsock ports and host CID, mobility modes, the checkpoint profile, host deadline authority, resources, devices and non-claims. Admission is pure, with typed refusals; `live_precopy` is refused at this revision. r[lattice.vm_profile.admission]
- Add `lattice.vm-attempt-binding.v1` (C2): profile identity, guest image, environment, envelope, attempt, fence, execution, workspace, requested mode and the original deadline. It contains no placement and no endpoint. Each crossed field has its own refusal code. r[lattice.vm_profile.binding]
- Use the existing domain-separated, length-prefixed BLAKE3 framing for C1 and C2 identities, and publish canonical bytes and identities as fixtures. r[lattice.vm_profile.identity]
- Give `lattice.implementation-attempt-run.v3` optional `vm_binding` and `vm_profile` fields. Both are absent from serialization when unset. When present, they require `network: false`, the request's own envelope, attempt and fence, an offered mode, and a credential-free forwarding broker of exactly this binding. Bound attempts report `vm-guest` facts in `inspect`. r[lattice.vm_profile.request]
- Give the broker profile an optional `forwarded` field (`lattice.model-broker-forwarded-upstream.v1`, C4). The guest broker forwards each admitted request over AF_VSOCK to CID 2, refuses any credential, keeps the direct route's settlement semantics, and blocks the worker before spawn when no host gateway is reachable. r[lattice.vm_profile.forwarded_upstream]
- Add `lattice model-gateway serve` on the host. It admits the profile and binding, serves the CH guest-initiated vsock socket only to the VMM's PID, and enforces the binding deadline on the host clock. For every request it asks Kiln's `lattice.vm-effect-authority.v1` (C3) through `KilnAuthorityGate`, a `ModelRequestGate`: `Admit` before the single upstream call, `Settle` after it. A lost settlement is `uncertain` and is never retried. r[lattice.vm_profile.gateway] r[lattice.vm_profile.effect_authority]

## Impact

- New: `src/vm_profile/{mod,core,model,tests}.rs`, `src/model_gateway/{mod,core,admission,shell,authority,tests}.rs` (`core` holds the C3/C4 records, `admission` the pure decisions; `mod.rs` roots keep the `src/` module count and filenames within the Octet baseline), `src/candidate_worker/broker/forwarded.rs`, `tests/test_model_gateway_process.rs`, `profiles/vm-execution/*.ncl`, `fixtures/vm-profile/**`, `docs/vm-execution-profile.md`.
- Changed: the broker (`broker.rs`, `admission.rs`, `wire.rs`, `host.rs`, `serving.rs`), `implementation_attempt/cli.rs` (the two request fields and one validation call), new `cli/vm/{mod,tests}.rs` (VM request validation and its tests) and `cli/inspect/mod.rs` (facts when bound).
- Shared, merged by the integration owner: `src/lib.rs` (two modules), `src/main.rs` (one subcommand), `Cargo.toml` (`nix` with `socket` for Unix targets; the lock is unchanged), and absent-field additions to existing test literals.
- Compatibility: v1 wire schemas that Kiln consumes are unchanged. A request or broker profile without the new fields keeps its bytes, identities and behaviour.

## Owner decisions recorded

- **OD-1:** Kiln consumes this work only through the `lattice` process boundary and the published fixtures. Nothing here assumes Kiln links new Lattice code.
- **OD-2:** single-host qualification uses a test authority only. Production placement waits for Aspen, and no local-lock fallback is introduced.
- **OD-3:** scripted model fixtures only; no live model credentials.
- **OD-5:** the #12 addenda stay with their producers and are out of scope here.

## Out of Scope

- The Kiln C3 server, the guest image, the CH vsock socket wiring and every Q2 qualification (S4).
- The guest workflow exchange over vsock (S2) and application checkpoints (S3).
- Cross-host, live-model and production-placement claims.
