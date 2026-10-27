## Phase 1: Binding core

- [x] [serial] Implement canonical binding projection over observation-source identity, workflow identity, trigger shape, and capability context, backed by BLAKE3. r[lattice.compose_surface.identity]
- [x] [serial] Implement pure admission over pane-observation DTOs with explicit schema bounds. r[lattice.compose_surface.admission]
- [x] [serial] Implement deterministic binding-plan decisions from supplied values only. r[lattice.compose_surface.plan]

## Phase 2: Shell and receipt

- [x] [serial] Add an injected adapter port for the Tile observation projection with exact revision binding. r[lattice.compose_surface.dependency]
- [x] [depends:lattice.compose_surface.receipt] Publish one metadata-only receipt per accepted activation through the shell. r[lattice.compose_surface.receipt]
- [x] [serial] Wire the host shell to compose only admitted plans and never execute effects. r[lattice.compose_surface.plan]

## Phase 3: Fixtures and evidence

- [x] [parallel] Add positive fixtures for key stability, valid admission, deterministic plans, and receipt issuance. r[lattice.compose_surface.verification]
- [x] [parallel] Add negative and boundary fixtures for missing fields, unknown fields, over-claims, and missing source revisions. r[lattice.compose_surface.verification]
- [x] [serial] Document the claim boundary and the recorded Tile contract. r[lattice.compose_surface.dependency]
- [ ] [serial] Run workspace, Clippy, formatting, Cairn, and Nix checks, then record evidence. r[lattice.compose_surface.verification]

## Implementation evidence and pending integration

- Native Cairn proposal, design, and tasks gates passed under the explicit
  repository policy before implementation and after the producer decision was
  recorded. These are structural admission checks, not runtime proof.
- `b3sum` independently checked the fixture canonical byte frames: 612-byte
  binding frame `ae290bfbb5090098b04fefeef43e2b9159f1c463c38ad5971db55716b19332a3`;
  369-byte receipt frame `7f6b771f92c3432cdf17e29e71cce31d838a87a0adb9305843cabe917af328f7`.
- Parent integration now declares the two Tile crates at the exact reviewed
  revision through authenticated GitHub SSH and refreshes Cargo's lock using
  `cargo update -p regex`; no sibling dependency or producer source copy is added.
- A disposable consumer compiled the actual adapter source, passed its three
  existing positive/negative tests and ran actual Tile projection/redaction plus
  Lattice request admission/rejection. This did not start or contact a daemon.
- Full application checks and a real daemon/consumer smoke remain unqualified
  because the independent component-runtime/verifier contracts are unresolved.
  The final verification task stays open. The explicit consumer remains
  `examples/compose-pane.rs`; operator commands and claim boundaries are in
  `docs/compose-surface-pane-binding.md`.
