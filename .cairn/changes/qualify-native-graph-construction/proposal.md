## Why

[Issue #20](https://github.com/OnixResearch/lattice/issues/20) asks Lattice to
qualify generated Nickel graphs through its existing admission and runtime, and
[issue #21](https://github.com/OnixResearch/lattice/issues/21) still lacks its
complete-module-closure, construction-origin, navigation and drift criteria.
Lattice now pins the published producers that supply those contracts, and the
consumer implementation exists, but no Lattice requirement, traceability or
criterion-by-criterion qualification binds it.

## What Changes

- Add `lattice.graph_construction.*` requirements for the selected captured
  construction profile, generated instances, reference agreement, exact local
  authority, zero-data control order, pre-dispatch rejection, drift
  invalidation, compatibility, runtime controls, module closure, origins and
  navigation, template/instance editing, in-flight snapshots, inert authoring and
  retained host qualification.
- Add focused regressions through production routes where the issue criteria
  were partial or missing: instance identities and attached facts, copy/drop
  rules, generation and expansion overflow, parameter/member/elaborator drift,
  same-graph authority, in-flight readmission, snapshot cancellation, shell
  result validation, orchestration budgets, and CLI/TUI navigation and run.
- Replace the sandboxed `construction-runtime` flake check, which cannot pass
  because Nix sandboxes do not map host root, with the store-installed
  `construction-runtime-tests` package and a host-run
  `construction-runtime-qualification` flake app that retains source, expanded
  graph, exact commands, results and negative controls.

## Consumer, producers and ownership

- Consumer: Lattice's captured local-shell construction profile, `lattice import
  --construction`, `lattice authoring` and the existing headless, snapshot and
  orchestration runtime routes.
- Producers: Choregraph `c3577352ee1e39391f5204ee234d991c7dbeb151`
  (construction/v3 projection, `choregraph.graph-construction.v1`) and
  nickel-eval `f0ea0d2a31d3482ddf4e575482f6ddf7a7e7d4e9` (captured-module
  evaluator; re-pinned by the parent from `2d1fb2b59f373967cc69e13ec289759d9cd4df46`).
- Maintenance owner: Lattice maintainers of `src/authoring/` and the
  construction profile. Producer contracts stay with Choregraph #6 and
  nickel-eval.
- The `lattice.source_authoring.*` requirements remain owned by
  `preserve-captured-workflow-source`.

## Non-claims

No semantic-equivalence, soundness, release, external-service or live-service
claim is made. Equal source, output or materialized hashes are not behavioral
equivalence. Registry names are not executable digests. The host-run app records
bounded observations for the selected store artifacts on one host; it is not a
sandboxed check, producer qualification or deployment receipt.
