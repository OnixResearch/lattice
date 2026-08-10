# Exact Choregraph Nickel lowering blocker — 2026-07-27

## Result

The machine cutover is complete, but direct production use of `choregraph-nickel` is blocked by exact dependency requirements.

The published candidate requires:

```text
choregraph-nickel -> blake3 =1.8.2
```

The selected Lattice Animus packages require:

```text
animus-adapters -> blake3 ^1.8.5
animus-router -> blake3 ^1.8.5
animus-runtime -> blake3 ^1.8.5
animus-steel -> blake3 ^1.8.5
```

Cargo cannot select one semver-compatible `blake3` release that satisfies both sets. The attempted exact dependency failed before compilation with a version-selection error.

## Bounded decision

Lattice does not edit the immutable Choregraph source, casually alter the reviewed Animus snapshot, weaken either version requirement, add a vendor fallback, or run the Choregraph CLI as a subprocess.

The existing Lattice Nickel evaluator and product decoder remain authoritative. Decoded runner values enter the published conversion-only `choregraph-seaglass` adapter and the Choregraph machine core.

The `lattice.choregraph_adoption.lowering` requirement and its combined task remain open until reviewed source releases align this dependency. This blocker does not reopen the removed graph implementation.

## Resolution — 2026-07-28

Choregraph revision `81b260da2256147ec82cb5dd44c0a0c1dcb7500d` uses a reviewed compatible BLAKE3 requirement. Lattice's generated lock selects BLAKE3 `1.8.5` for Choregraph and Animus.

Lattice now passes decoded workflow values to the pure `choregraph-nickel` lowerer. No dependency override, subprocess, sibling path, vendor copy, or fallback was added.
