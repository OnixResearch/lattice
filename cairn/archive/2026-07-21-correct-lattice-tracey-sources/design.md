## Context

Lattice had only `cairn-policy/generated/cairn-policy.json`; it had no repository-owned Nickel policy source or `.config/tracey/config.styx`. The generated default profile was inherited from Cairn and scanned `crates/` and `tools/`. Those directories do not own Lattice's accepted implementation and verification markers, so the compatibility command reported `referenced: 0` for all 692 accepted requirements.

The prior retry archive described the fallback profile as a local `.config/tracey/config.styx`. That path does not exist. This change retains the historical archive and records the corrected root cause and replacement evidence here.

## Decisions

### Decision: Own the policy in typed Nickel

**Choice:** Import Cairn's current policy contracts and default source at revision `19fc6793720d5fea0185d4d3cab1626ddbc67a40`, then customize only the default traceability evidence roots and regenerate the checked JSON through `cairn policy export`.

**Rationale:** Nickel remains the reviewable source of truth, the existing generated-policy consumer contract remains stable, and `policy export --check` detects drift. Importing the complete policy source avoids a circular generated-JSON dependency that Cairn's Nickel export boundary correctly rejects.

### Decision: Scan bounded repository-owned roots

**Choice:** Keep accepted requirements under `cairn/specs` and scan Rust markers under `src`, `tests`, `examples`, and `scripts`, plus Nickel markers under `examples` and `cairn-policy`.

**Rationale:** These roots contain Lattice-owned implementation, tests, examples, tools, and policy. Excluding `vendor-cargo-deps`, `target`, and generated JSON prevents imported or generated text from manufacturing local traceability evidence. The Nix source filter explicitly retains `cairn-policy` so compile-time policy tests exercise the same checked files in isolated builds.

### Decision: Expose rather than suppress real debt

**Choice:** Preserve the exact `impl`, `verify`, `depends`, and `related` marker verbs and retain failure when accepted requirements are missing references or evidence markers are dangling.

**Rationale:** Success for this bounded change is correct source discovery, not a false green coverage claim. The corrected command discovers existing markers and then fails on the repository's real linkage debt, which can be drained separately.

### Decision: Pair positive and negative policy checks

**Choice:** Test the exact generated source set and marker verbs, exclude the stale `crates`/`tools` fallback roots, and require an unsafe `../src` fixture to be rejected during policy export.

**Rationale:** This proves both intended admission and fail-closed path validation without relying on ambient filesystem traversal or weakening Cairn's policy validator.

## Risks / Trade-offs

- The copied Cairn policy contracts and defaults are a reviewed snapshot and require explicit refresh when Cairn's policy schema changes; the source revision and freshness command make that dependency visible.
- Correct source discovery increases visible missing and dangling counts. Those findings are existing traceability debt, not regressions introduced by the profile.
- This change does not prove behavioral correctness, test adequacy, complete requirement coverage, or release readiness.
