# Post-archive validation

Lattice pins Cairn revision `5e1fe08d62149b286ff583eb056e655fe92e3cf1` and uses its checked repository policy overlay.

The local policy export is fresh. Cairn validation passes while preserving three unrelated active changes.

The profile rejected planned sync before completion evidence existed. The retained receipt has BLAKE3 `5bca52837771e688f0b90c64c9bd929b43854824ac679016d74c776cf3196162`.

The fresh-state task-gate receipt has BLAKE3 `8e166da60a0dd6903ff164edc84aa8cd17537d79d1308bc992f180e038e339ed`.

The repository Octet check receipt has BLAKE3 `ba8fae58b95010e55a3ddd16f0f0a3bf6d3b1e21b82c84de2c8cec034c7d9b11`.

Sync mutation manifest `2f4425a174cf22a9853d0ec8bc440d87736181a0c290ee33ec8c59e42a15b205` accepted the requirement.

Archive mutation manifest `f1aa7349f006e607be6ea34f90020e57fc05145af826bcac3b5306661213539d` retained all referenced evidence.

Positive and negative runner tests pass. Focused no-dependency Clippy passes with warnings denied.

The local Octet Nix gate passes with no new or stale baseline findings. Its raw output remains warning-only and is not strict Octet acceptance.

The remote Octet route hit the existing `rust-src` transfer mismatch. Direct full-scope Clippy also fails in unchanged vendored sources.

Repository traceability still reports existing unrelated debt. The new `lattice.lifecycle.outcome_machine_profile` requirement is not missing or dangling.

These checks do not establish workflow correctness, external effect truth, or Lattice release eligibility.
