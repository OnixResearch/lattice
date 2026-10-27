# Vendored source snapshots

These snapshots keep product builds independent of sibling checkout paths. `scripts/sync-vendored-sibling.rs` imports only explicitly selected Git archive paths and writes a `SOURCE` file with the repository and exact revision.

- `rats/source/` comes from Rats revision `ef26e13b2e9cbcb4a0c902f9caff641b6d013867`.
- `animus/` comes from Animus revision `f1662a8ed290b170c3986d2ddf2909b36d7e5fdc`.
- `host-control/` comes from Host Control revision `7735faf3d8895b0bfa89f563184c4321b3b144a1`.
- `basalt/` comes from Basalt revision `b34d6ac27ad396530fb799e79223acf71112ee0f`.
- `hardened-wasmtime-runtime/` comes from shared runtime revision `1d642fa5cba7ce8a6d8f127f28ec6d8505842378`.
- `mantle/` comes from Mantle revision `dcf4b51b3210f25f08e96d4e7bc1653d2fbcd240`.

The snapshots preserve source provenance. They do not transfer correctness claims.

Lattice maintainers own these import selections and their consumer adapters;
the producer repositories retain their runtime, verification and authority
contracts. `SOURCE` records the exact archive path selection. Refresh through
the sync script with those paths, never by patching an imported file. The
native workspace manifests remain intact and excluded from Lattice's workspace;
the root package stays an implicit workspace member so Cargo does not admit a
second native workspace root accidentally. No integration is feature-disabled.

The native correction commits above are local immutable source cohorts, not a
claim of remote publication or anonymous acquisition. Their imported bytes are
available to standalone builds. Refreshing them requires the exact producer Git
objects; it must not substitute a floating branch or a sibling dependency path.

`rats/Cargo.toml` is Lattice-owned import metadata outside the immutable
`rats/source/` snapshot. It supplies the historical MIT workspace default and
the same inherited dependency values the sparse Rats manifests previously took
from Lattice's root workspace. Lattice's owned workspace now uses AGPL; this
separate import workspace prevents that default from relabeling Rats. The sync
script carries this checked wrapper to the destination parent on refresh and
refuses to overwrite a different existing workspace. Imported source bytes and
their revision are unchanged; the wrapper itself is AGPL-owned authoring.

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
