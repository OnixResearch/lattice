# Cairn policy provenance

`contracts.ncl` and `upstream-default.ncl` are exact snapshots from Cairn `main` revision `317d3ff648f01a5ea1572e6126f4dd71bc7afa99`, the same revision as the flake input. That revision reads consumer-selected release gates and refuses a malformed list. Its policy loader also rejects the earlier `5e1fe08d62149b286ff583eb056e655fe92e3cf1` snapshots, which lack the generated-workflow `command_catalog`, `entry` and `readiness` fields. The snapshots' imports, `lifecycle-store-contracts.ncl` and `nominal-evidence-identities.ncl`, are unchanged at that revision.

`default.ncl` overlays Lattice traceability roots, declares Lattice's maintained
release gate list in `release_readiness_policy`, and keeps the upstream
lifecycle, workflow, receipt, and validation policy.

`traceability-contracts.ncl` adds Lattice-owned component-wise admission for those roots. It rejects absolute, empty, dot, parent-traversing and non-normalized components without changing either upstream snapshot.

`release-selection-contracts.ncl` adds Lattice-owned admission for the release
selection extension: a non-empty list of unique, non-empty command strings plus
explicit completion output paths. `default.ncl` applies this contract to the
authored policy; it is not test-only metadata.

The snapshot supplies policy structure only. It does not transfer Cairn correctness, Lattice correctness, release authority, or evidence authority.
