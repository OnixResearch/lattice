## Context

The repository currently exposes `n8nclone` through Cargo package/crate/bin names, Nix outputs, command examples, generated text, default state paths, environment variables, logs, and tests. The same text also appears in two categories that are not branding: stable Cairn requirement IDs/evidence links and a versioned webhook signature domain. A blind repository-wide replacement would break persisted state, invalidate webhook signatures, and sever traceability to accepted requirements and archived receipts.

## Decisions

### Decision: Make Lattice canonical at public build and operator surfaces

**Choice:** Rename the Rust package, library crate, executable, Nix package/check output, documentation, examples, and current user-visible strings to `lattice`/Lattice. The old executable is not emitted as a second primary product.

**Rationale:** One canonical identity prevents split installation paths and makes build metadata, command documentation, and runtime output agree.

### Decision: Separate compatibility facts from filesystem discovery

**Choice:** Pure selection functions decide canonical versus legacy data directories and store filenames from explicit facts. Thin shells read environment variables and path-existence facts. `LATTICE_DATA_DIR` wins, `N8NCLONE_DATA_DIR` remains a fallback, `~/.lattice` is canonical, an existing `~/.n8nclone` remains discoverable, and an existing `n8nclone.redb` remains usable when no canonical database exists.

**Rationale:** Existing operators retain state without hiding ambient filesystem reads inside policy logic, and the compatibility behavior can be tested without standing up a home directory.

### Decision: Keep versioned and evidence identities stable

**Choice:** Preserve `n8nclone-webhook-signature:v1`, accepted `n8nclone.*` Cairn requirement IDs, their implementation/test markers, and all prior `cairn/archive/**` content. Add new rename requirements under `lattice.project_identity.*`.

**Rationale:** The webhook domain is cryptographic wire input and changing it would invalidate existing signatures. Cairn IDs are durable linkage keys, not presentation labels; rewriting them would break historical receipts and traceability.

### Decision: Bound repository-host mutation

**Choice:** Rename the local checkout and parent workspace ignore entry after the child repository archive commit. Do not rename the GitHub repository or push new commits without explicit push authorization.

**Rationale:** Local source identity is part of this implementation. Remote repository mutation and history publication are separate authority-bearing operations.

## Risks / Trade-offs

- The old executable name stops being built; scripts invoking it must switch to `lattice`.
- Compatibility leaves intentional `n8nclone` strings in stable protocol, state, evidence, and historical surfaces; a zero-match grep is therefore not a valid completion criterion.
- Crates.io publication of `seaglass-execution-graph` remains blocked until Cargo registry credentials are supplied; the rename must not fabricate publication success.
