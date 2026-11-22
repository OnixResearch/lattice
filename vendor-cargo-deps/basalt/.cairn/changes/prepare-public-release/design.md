## Context

Basalt's crate, policy fixtures, and reviewed UCAN snapshot already resolve from
anonymous sources: crates.io, public GitHub repositories, and pinned
`seed.radicle.garden` objects. The remaining release-readiness gaps are
presentation and hygiene rather than product behavior: one internal scratch path
in a research report, a tracked editor
backup, instructions that assume sibling checkouts, missing security reporting
guidance, and a CI runner that runs out of disk before the Nix build.

## Decisions

### Decision: Keep immutable evidence and vendored snapshots unchanged

**Choice:** Report `/home/` paths inside `.cairn/archive/**` and the
`git@github.com:` record inside `vendor/ucan/vendor/verified-logic-revision.txt`
as residual exposures instead of editing them.

**Rationale:** Those bytes are archive evidence or a reviewed vendored snapshot.
Editing them would break content identities and the repository's own provenance
rules. None contains a credential, and the vendored record is not a build input.

### Decision: Keep the optional consumer runner local-only

**Choice:** Remove private consumer names and paths from the README, but retain
the local runner and its optional manifest path in release tooling.

**Rationale:** The crate build, package contents, and flake check do not use the
consumer fixture. The verifier already defers when the checkout is absent. Moving
or renaming the local fixture would change release-tool behavior and lifecycle
requirements without improving anonymous buildability. The runner's Cargo patch
key stays `ssh://git@github.com/OnixResearch/basalt.git` because a `[patch]` key
must equal the source URL consumers declare; it is not a Basalt build input.

### Decision: Fix CI disk pressure in the workflow, not by changing pins

**Choice:** Delete unused preinstalled runner toolchains before installing Nix
and keep the existing `contents: read` permission.

**Rationale:** The observed failure is runner disk exhaustion while building
Nickel, not a private input or trust boundary. A standard cleanup step is the
smallest change and preserves the fork-safe `pull_request` model.

## Risks / Trade-offs

- The CI cleanup step cannot run locally; its effect is unverified until the
  parent pushes and observes the GitHub Actions run.
- The residual archive and vendored references remain visible in public history.
  They expose workspace paths and an SSH-style remote record, not credentials or
  reachable private services.
- Local Cargo artifacts from this work must use the operator-owned target
  directory outside the worktree.
