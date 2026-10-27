## Context

The reported repository checks already establish three concrete failures: accepted-spec marker placement disagrees with the maintained shape validator; two executor warning identities changed while accepted warning classes remain; the admitted traceability profile omits build metadata and leaves maintained behavior unlinked. Existing debt inventory describes an older twelve-gap snapshot and must not be presented as current evidence.

## Decisions

### Decision: Preserve requirement meaning and gate strictness

**Choice:** Normalize accepted Markdown to the repository's existing body-marker/GIVEN-WHEN-THEN form, preserving every stable ID and requirement/scenario meaning. Keep real shape validation and fail-closed missing/dangling coverage.

**Rationale:** Deleting a check, excluding a failing specification, or relabeling unsupported requirements would hide the reported failures rather than resolve them.

### Decision: Review evidence at the implementation or assertion

**Choice:** Review each missing requirement against maintained source and observable regression assertions. Add markers only at supporting implementation or behavior checks. Use `impl` for implementation facts and `verify` only for checks that assert the claim. Preserve explicit non-claims. Extend evidence discovery only to exact maintained build files and exclude vendor snapshots, archived changes, generated debt rows, and accepted-spec prose as implementation evidence.

**Rationale:** Coverage is declared linkage, not semantic proof. Correct discovery can expose existing evidence, but comments and inventory classifications cannot manufacture missing behavior.

### Decision: Migrate identities without admitting new warning facts

**Choice:** Require complete successful reference and candidate raw observations under the accepted tool/profile/config cohort. Compare every normalized warning fact and raw multiplicity before applying Octet's canonical first-per-stable-ID selection and comparing the accepted baseline facts. Exercise paired acceptance/rejection cases, including colliding runner identities, incomplete observations and changed warning facts. Generate the final baseline only after all source edits finish; retain the admitted observations for review.

**Rationale:** Octet's baseline keeps one representative per stable ID, while a compiler span can produce several warning messages with that ID. Baseline rows alone cannot establish raw-warning conservation. Complete raw comparison prevents canonicalization from hiding new debt; final canonical comparison prevents changes to accepted representatives.

### Decision: Verify the producer before refreshing the consumer receipt

**Choice:** Finish independent source slices before running shared format, test, lint, lifecycle, inventory, and Nix gates. Reuse the actual local Kiln HTTP/SQLite/Lattice Unix execution path, then inspect operator deployment inputs before any live GitHub mutation.

**Rationale:** A local provider fixture cannot establish a deployed credential, ingress, or untrusted-worker isolation boundary.

## Risks / Trade-offs

- Review may find a claimed requirement lacks supporting behavior. That requires a real bounded implementation/regression or an explicit blocker; no unsupported marker may be added.
- Markers can change Octet content identities without changing warning semantics. Baseline migration must follow final source edits, not run concurrently with them.
- Analyzer overflow fixtures must consume the admitted request before exiting. A stdout-only fast exit can fail stdin delivery before output classification; retain exact timeout/truncation outcomes and required-parent-failure assertions rather than changing production failure policy.
- Expression-command fixtures use expression-derived dependencies without adding an unrelated piped-stdin edge to a command that does not consume stdin. Retain the exact stdout interpolation and dependency-order assertions; do not suppress bounded process input-delivery failures.
- Historical review fixtures retain their original coverage cohort. After reviewing the supporting executable assertions, regenerate changed evidence identities with the maintained inventory core, including its policy and report hashes; preserve all requirement IDs, classifications, rationales, non-selected rows and coverage counts rather than weakening byte binding.
- Every Nix entrypoint that runs lifecycle integration tests must select the pinned native Cairn binary and exact Nickel executable. The developer Cairn wrapper may require a Nix daemon and is not a sandbox test dependency.
- The prior checkout dataset exhausted quota. Reuse preserved verification output or the shared Cargo target, and keep new large scratch outputs on `/tmp`; do not remove unrelated caches or source.
- If an unrelated workspace job holds Cargo's global package-cache lock, preserve that job. Host checks may use isolated offline Cargo state with cache garbage collection disabled while reusing the pinned source cache.
- Existing user changes and independent repository histories must remain intact. Live operation is limited to an admitted deployment and exact repository authority.
