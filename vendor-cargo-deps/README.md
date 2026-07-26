# Vendored source snapshots

These snapshots keep product builds independent of sibling checkout paths. `scripts/sync-vendored-sibling.rs` imports only explicitly selected Git archive paths and writes a `SOURCE` file with the repository and exact revision.

- `rats/source/` comes from Rats revision `55e66180c1c1227d9b6592d2e058a35147eb3b22`. Its parent published revision `f6715ae7eb6a8b0bf4a2af08d888fd01e7537f2a` omitted `ratcore/src/pointer/routing/result.rs` from source archives because of an over-broad ignore rule. Revision `55e6618` tracks that required module and keeps `result.rs` files in source distributions.

The snapshots preserve source provenance; they do not transfer Seaglass or Rats correctness claims. Re-run focused consumer tests after every synchronization.
