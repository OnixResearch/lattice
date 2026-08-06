# Vendored source snapshots

These snapshots keep product builds independent of sibling checkout paths. `scripts/sync-vendored-sibling.rs` imports only explicitly selected Git archive paths and writes a `SOURCE` file with the repository and exact revision.

- `rats/source/` comes from Rats revision `ef26e13b2e9cbcb4a0c902f9caff641b6d013867`.
- `animus/` comes from Animus revision `f1662a8ed290b170c3986d2ddf2909b36d7e5fdc`.
- `host-control/` comes from Host Control revision `7735faf3d8895b0bfa89f563184c4321b3b144a1`.

The snapshots preserve source provenance. They do not transfer correctness claims.

## Refresh the snapshots

Refresh Host Control before Animus:

```bash
scripts/sync-vendored-sibling.rs \
  --source ../host-control \
  --source-url ssh://git@github.com/OnixResearch/host-control.git \
  --revision 7735faf3d8895b0bfa89f563184c4321b3b144a1 \
  --destination vendor-cargo-deps/host-control \
  --name host-control \
  --path Cargo.toml \
  --path crates/host-control-core
scripts/sync_animus.rs ../animus vendor-cargo-deps/animus
scripts/sync_rats.rs PATH_TO_RATS_REPOSITORY vendor-cargo-deps/rats/source
```

Run the focused consumer tests after each synchronization.
