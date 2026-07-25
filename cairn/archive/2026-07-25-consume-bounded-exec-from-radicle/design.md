## Context

Lattice currently names Bounded Exec in three coupled source surfaces:

- Cargo's exact Git dependency;
- the Nix flake input used to provide the source without Cargo network access in sandboxed builds;
- Crane's exact Cargo source string used to substitute that flake input during vendoring.

Changing only one surface would produce lock drift, duplicate source identities, or a build that works locally but fails in Nix. The pilot therefore treats those declarations and generated locks as one source-identity contract.

Completion begins only after the accepted Bounded Exec publication receipt binds concrete `onix-core` node/HTTPS deployment evidence and OnixOS multi-seed admission. It ends when Cargo and Nix resolve the accepted Radicle RID to the same existing Bounded Exec object with no GitHub source or fallback for that dependency. A documentation-only URL change, unlocked branch, path override, or GitHub-backed lock entry is false completion.

## Decisions

### Decision: Keep the dependency revision fixed

**Choice:** Retain `29dac88ecded94457572db3fdfaaaab95fa91525` while changing the Git source to the accepted Radicle-backed HTTPS endpoint.

**Rationale:** The pilot tests forge and transport migration. Combining it with a source upgrade would prevent attribution of behavioral or build differences.

### Decision: Use HTTPS Git as the package-manager adapter

**Choice:** Cargo and Nix use a stable HTTPS Git endpoint backed by the canonical RID. Developer Radicle remotes may use `rad://`, but manifests do not require `git-remote-rad`, a running local node, or ambient profile state.

**Rationale:** Cargo, Nix evaluators, sandboxed builds, and remote builders understand ordinary Git transport. The RID remains the canonical identity recorded by the producer receipt; the HTTPS endpoint is replaceable transport.

### Decision: Regenerate, never hand-edit, locks

**Choice:** Update source declarations first, then regenerate `Cargo.lock` with Cargo and `flake.lock` with a targeted Nix lock operation. Review the generated source URL, revision, and content identities before acceptance.

**Rationale:** Both lock formats are tool-owned compatibility surfaces. Manual edits can create internally inconsistent source identity and are prohibited.

### Decision: Validate source agreement before compilation

**Choice:** Add a pure source-identity validator over explicit manifest and lock facts. It checks the accepted RID/endpoint identity, exact revision agreement, expected package names, and absence of a Bounded Exec GitHub fallback. File reads and report rendering remain in a thin repository-check shell.

**Rationale:** Source mismatch should produce stable diagnostics before Crane, Cargo, or Nix fails later with a less precise vendor or fetch error.

### Decision: Outage drills stay separate from deterministic CI

**Choice:** Deterministic repository checks validate source declarations and locks without intentionally breaking network services. `onix-core` performs concrete node deployment and outage operations; the OnixOS pilot validates their catalog alignment and supplies BLAKE3-bound acceptance references for Lattice.

**Rationale:** A normal `nix flake check` should not depend on deliberately stopping shared infrastructure. Live availability evidence has a different owner and lifecycle.

## Functional Core and Shell

The pure core consumes typed values for Cargo source, Nix source, Crane substitution source, lock revisions, accepted producer identity, and forbidden fallback hosts. It returns a sorted diagnostic set and accepted source identity. It performs no file, Git, Nix, network, clock, environment, or process access.

The shell reads bounded repository files, supplies parsed facts, and renders the result. Existing Cargo/Nix commands remain responsible for lock generation and source acquisition. No runtime Lattice code gains Radicle node or network behavior.

## Validation

Positive validation proves:

- Cargo, Nix, Crane, and both generated locks agree on the accepted source and exact revision;
- the fetched source contains the expected crates and AGPL license metadata;
- focused local-process success, failure, timeout, truncation, cancellation, and teardown tests retain their outcomes;
- each admitted `onix-core` seed path can supply the exact object under the OnixOS acceptance drill.

Negative validation covers different RIDs or endpoints, revision mismatch, stale GitHub lock entries, hidden path overrides, missing crates, altered source content, malformed producer evidence, unavailable Radicle paths, and attempted GitHub fallback.

## Risks / Trade-offs

- The HTTPS gateway becomes a package-manager availability dependency even though the RID remains the canonical Radicle identity.
- Cargo lock source text and Nix lock representation change despite identical source bytes; review must distinguish transport metadata from code changes.
- GitHub may remain necessary for unrelated third-party dependencies. This change claims independence only for the Bounded Exec source.
- Passing checks proves source agreement and retained tested behavior, not Radicle protocol correctness, Bounded Exec correctness, host sandboxing, or Lattice release readiness.
