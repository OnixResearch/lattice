## Why

Kiln #70 needs an admitted parent to derive a narrower child grant for another holder, and
Kiln #75–#79 need capability-scoped source operations. The frozen
`kiln://implementation/v1/` resource frames the holder first. UCAN coverage is a byte
prefix, so a ceiling held by one holder can never cover a request of another, and only
root-wide grants delegate. Basalt also has no source-view operations
(`src/kiln.rs` operation list). The owner approved Basalt B1 (attenuable implementation
framing plus a delegate rule) and B2 (`kiln://source/v1/` operations) in the Kiln
authority plan §7.

## What Changes

- Add `ImplementationFraming::{V1, V2}` on `kiln::Request` (serde default `v1`). V2 frames
  `audience/generation/project/work/work_version/attempt/fence/target/digest/candidate`
  without the holder. `ImplementationScope::prefix` builds ceilings.
- Add a chain rule for attenuable families in `check_roots`. Each intermediate delegator
  must hold an explicit, uncaveated `kiln/delegate` (implementation) or
  `kiln/source/delegate` (source) capability over every covering grant it issued. Covering
  grants must stop on a segment boundary. The depth must stay within the new
  `Verifier::max_delegation_depth`.
- Keep v1 byte-identical and keep its trusted-root-only chain rule. A golden fixture pins
  the bytes.
- Add `SourceOperation` (discover, read, history, prepare, modify, create, delegate,
  result), `SourceRequest`, `SourceScope`, `SourceSession` and
  `Verifier::{source_enforcement, verify_source, revalidate_source}`. Path scope is framed
  per component with an `e/` or `s/` terminal marker.
- Add eight `EnforcementAction::KilnSource*` variants and their Nickel contracts. Let the
  nine implementation contracts admit the v2 prefix.
- Regenerate `contracts/generated/policy.json`, refresh the policy BLAKE3 identity pins and
  regenerated authority-core and UCAN-semantics fixtures, and update
  `docs/kiln-authority-adapter.md`.

## Impact

- **Library**: `src/kiln.rs`, new `src/kiln/framing.rs` and `src/kiln/source.rs`,
  `src/ucan_enforcement/types.rs`.
- **Breaking for struct-literal callers**: `kiln::Request` gains `framing`, and
  `kiln::Verifier` gains `max_delegation_depth`. Kiln updates both on re-import. The
  serialized v1 request form is unchanged.
- **Policy**: `contracts/policy.ncl`, its generated artifact, the policy identity pins in
  `src/ucan_semantics_identity.rs`, `contracts/ucan-profiles`, `contracts/ucan-semantics`,
  `contracts/authority-core/fixtures/base.ncl`, their 19 generated fixtures and two docs.
- **Tests**: `tests/ucan_enforcement/kiln.rs` (new fields, v1 golden),
  `tests/ucan_enforcement/kiln_attenuation.rs` (new), `tests/policy_artifact.rs`, and
  framing unit tests.
- **Consumers**: Kiln S6 consumes the v2 grammar and delegate rule; S5 and S7 consume
  source operations. No Kiln or Lattice code changes here.
