## Phase 1: Public build closure

- [x] [serial] Move the Tile compose shell and example into an excluded workspace with its own lockfile, and remove private Tile dependencies from the default runtime. r[lattice.public_release.build.private_source]
- [x] [serial] Switch direct and transitive public source acquisition to anonymous HTTPS while preserving exact revisions. r[lattice.public_release.build.anonymous]
- [x] [serial] Add public Cargo package metadata and CLI version output. r[lattice.public_release.package.metadata]

## Phase 2: Release and operator surface

- [x] [serial] Add the security policy, contribution guide, changelog, documentation index, and public status corrections. r[lattice.public_release.docs.trust]
- [x] [serial] Add the server deployment guide and systemd unit with local-service non-claims. r[lattice.public_release.deploy.local]
- [x] [serial] Add the release process and tagged release workflow with checks, binary archive, and checksums. r[lattice.public_release.artifact]
- [x] [serial] Add the go-to-market plan, proof-point links, demo script, and launch checklist. r[lattice.public_release.launch]

## Phase 3: Verification

- [x] [serial] Regenerate the Cairn policy and verify the release input manifest covers the new surfaces. r[lattice.public_release.evidence]
- [x] [serial] Run `cargo metadata --locked`, default `cargo check`, policy export, and flake evaluation. r[lattice.public_release.build.anonymous]
- [x] [serial] Run the compose workspace tests with authorized Tile access. r[lattice.public_release.build.private_source]
- [x] [serial] Smoke `lattice --version`, `lattice --help`, and the effect-free import/export path. r[lattice.public_release.package.metadata]
- [x] [serial] Record the observed release audit, remaining external actions, and non-claims. r[lattice.public_release.evidence]

## Phase 4: Remaining release actions

- [x] [serial] Resolve the stale Octet baseline without bulk-accepting unreviewed findings, then run `octet-precommit-check` and `nix flake check` green on the release commit. r[lattice.public_release.evidence]
- [ ] [serial] After explicit release authorization, tag the checked commit and publish the binary archive through the release workflow. Bounded Exec is now vendored from the exact public GitHub tree at `29dac88ecded94457572db3fdfaaaab95fa91525`; the remaining action is the tag move and workflow publication. r[lattice.public_release.artifact]
