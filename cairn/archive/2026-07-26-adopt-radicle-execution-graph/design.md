## Context

Lattice depends on `vendor-cargo-deps/seaglass-execution-graph` and retains a repo-owned sync path. The standalone repository records the copied Seaglass source and exposes package `execution-graph-core`. Its reviewed pre-publication object is `03736f1ec46c377ff86b451260ad68aa70ff3b0b`.

The external repository owns deterministic graph mechanics only. Lattice maps product workflows into that model and retains every effectful or product-specific decision.

## Success Contract

Completion requires an accepted Radicle publication receipt, exact Cargo/Nix source alignment, full positive and negative dual-run parity, removal of duplicate source, and tested rollback. Merely changing `Cargo.toml`, using `../execution-graph`, or retaining an automatic vendor/GitHub fallback is false completion.

## Decisions

### Require producer publication first

Cutover is blocked until `execution-graph` records a canonical RID, reviewed authority policy, unchanged required Git object, source archive BLAKE3, admitted endpoint, explicit license, and negative no-fallback evidence.

### Preserve imports through a temporary alias

Cargo may alias package `execution-graph-core` to the existing dependency key while migration is active. The alias does not transfer Seaglass product ownership into the standalone repository and is removed when call sites adopt the canonical crate name.

### Align Cargo and Nix exactly

Both source paths name `https://git.onix.computer/<rid>.git` and one exact `rev`. No floating branch, tag-only source, GitHub URL, or sibling path is admitted. `Cargo.lock` and `flake.lock` are generated only by Cargo and Nix; `flake.lock` is never edited manually.

### Dual-run before removal

A temporary test adapter evaluates identical node/edge and lifecycle facts through vendor and external implementations. It compares admitted diagnostics, deterministic order, dependency queries, parallel groups, readiness, transitions, resume, critical path, and bottlenecks. Every difference is reviewed before cutover.

### Keep product authority in Lattice

Typed ports, node configs, values, effects, caching, retries, schedules, triggers, orchestration, persistence, run records, artifacts, diagnostics, and evidence remain Lattice-owned. The external core does not schedule or execute work.

### Roll back through reviewed source state

There is no runtime fallback. Rollback means reverting the dependency declarations, generated locks, adapter changes, and vendor removal together to the prior reviewed commit.

## Validation

Positive fixtures cover current admitted workflows and lifecycle behavior. Negative fixtures cover malformed nodes/edges, cycles, missing endpoints, terminal mutation, source mismatch, Cargo/Nix mismatch, unknown RID, missing revision, and attempted GitHub or sibling fallback.

## Claim Boundary

Passing cutover checks proves only selected source alignment and graph-decision compatibility. It does not prove workflow semantics, scheduler fairness, effect safety, persistence correctness, external availability, or release readiness.

## Risks

- Source layout or crate-name changes can create duplicate packages. Cargo tree and source checks must reject this.
- A parity fixture can miss product call paths. The inventory must bind every current adapter entry point.
- Vendor removal weakens offline acquisition. Nix/Cargo cache policy remains separate; no hidden fallback is added.
