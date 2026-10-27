## Context

Cairn currently hardcodes its self-host completion/corpus fixtures and repository gate list. Lattice must consume a typed producer-neutral release selection and keep policy identity explicit.

## Decisions

### Decision: Observe native checks, never mint passing results

**Choice:** A maintained Rust input producer runs native Cairn validation under explicit Lattice policy, retains stdout/status and exact native executable identity, and derives completion input status only from the observed native verdict. It refuses malformed output, records non-passing status truthfully, and does not present those bounded observations as all release gates.

**Rationale:** A copied upstream self-test receipt is not Lattice evidence.

### Decision: Consumer-owned corpus and replay

**Choice:** Maintain canonicalization vectors for Lattice workflow-contract values (ordering, null/absence and sequence semantics). Select actual status/validate/MCP/corpus/readiness replay cases and scoped inventory; use repeated in-place release cases and cross-root status/validate. External gate list remains scripts/check.rs, octet-precommit-check and documented Nix flake check. Parent exports the typed policy and admits exact public producer revision.

**Rationale:** Lattice does not promise Cairn self-host legacy migration fixtures, and required replay must execute actual selected surfaces without inventing fixtures.

## Risks / Trade-offs

- New Cairn source must pass native consumer behavior tests and be published at an immutable revision before consumer pinning.
- Input generation follows final policy/source edits; replay runs only on the integrated final tree.
- Optional authentication/provenance/semantic checks retain explicit skipped/disabled reports; onix_release_ready may truthfully remain false.
