# Cairn policy provenance

`contracts.ncl` and `upstream-default.ncl` are exact snapshots from Cairn revision `5e1fe08d62149b286ff583eb056e655fe92e3cf1`. The flake input is pinned to Cairn revision `47ea1b867c44b026c030b535f9a1041950c482a0`, which adds consumer-selected release gates without changing those policy snapshots.

`default.ncl` overlays Lattice traceability roots, declares Lattice's maintained
release gate list in `release_readiness_policy`, and keeps the upstream
lifecycle, workflow, receipt, and validation policy.

`traceability-contracts.ncl` adds Lattice-owned component-wise admission for those roots. It rejects absolute, empty, dot, parent-traversing and non-normalized components without changing either upstream snapshot.

The snapshot supplies policy structure only. It does not transfer Cairn correctness, Lattice correctness, release authority, or evidence authority.
