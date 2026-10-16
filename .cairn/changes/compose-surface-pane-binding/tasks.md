## Phase 1: Binding core

- [ ] [serial] Implement canonical binding projection over observation-source identity, workflow identity, trigger shape, and capability context, backed by BLAKE3. r[lattice.compose_surface.identity]
- [ ] [serial] Implement pure admission over pane-observation DTOs with explicit schema bounds. r[lattice.compose_surface.admission]
- [ ] [serial] Implement deterministic binding-plan decisions from supplied values only. r[lattice.compose_surface.plan]

## Phase 2: Shell and receipt

- [ ] [serial] Add an injected adapter port for the Tile observation projection with exact revision binding. r[lattice.compose_surface.dependency]
- [ ] [depends:lattice.compose_surface.receipt] Publish one metadata-only receipt per accepted activation through the shell. r[lattice.compose_surface.receipt]
- [ ] [serial] Wire the host shell to compose only admitted plans and never execute effects. r[lattice.compose_surface.plan]

## Phase 3: Fixtures and evidence

- [ ] [parallel] Add positive fixtures for key stability, valid admission, deterministic plans, and receipt issuance. r[lattice.compose_surface.verification]
- [ ] [parallel] Add negative and boundary fixtures for missing fields, unknown fields, over-claims, and missing source revisions. r[lattice.compose_surface.verification]
- [ ] [serial] Document the claim boundary and the recorded Tile contract. r[lattice.compose_surface.dependency]
- [ ] [serial] Run workspace, Clippy, formatting, Cairn, and Nix checks, then record evidence. r[lattice.compose_surface.verification]
