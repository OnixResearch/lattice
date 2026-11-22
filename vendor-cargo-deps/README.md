# Vendored source snapshots

These snapshots keep product builds independent of sibling checkout paths. The
import scripts copy only explicitly selected Git archive paths; Basalt, Host
Control, hardened runtime and Mantle also write a `SOURCE`
file with the repository and exact revision, while Rats and Animus record their
revisions below.

- `rats/source/` comes from Rats revision `ef26e13b2e9cbcb4a0c902f9caff641b6d013867`.
- `animus/` comes from Animus revision `e6f27948b19ea2e1bcb67428ca2e12f7acb6e558`, as recorded in `animus/source.ncl`.
- `host-control/` comes from Host Control revision `7735faf3d8895b0bfa89f563184c4321b3b144a1`.
- `basalt/` comes from Basalt `main` revision `fac0d7f1fd7ac936ca4d3f251f3287666e93d4bb`, which is published on GitHub and Radicle. It builds on `5e0f1f4cad925a50f7dd5cdbcae05a04e7b92463`, which supplies Kiln implementation v2 framing, source v1 operations and the delegate rule, UCAN `fca62a6e` and the ring RS256 OIDC check. `fac0d7f1` adds the holder series: OIDC holder delivery bound to the exact canonical Mantle request, one exchange per OIDC token, and shared canonical Mantle signing.
- `hardened-wasmtime-runtime/` comes from shared runtime revision `1d642fa5cba7ce8a6d8f127f28ec6d8505842378`.
- `mantle/` comes from Mantle revision `dcf4b51b3210f25f08e96d4e7bc1653d2fbcd240`.
- `bounded-exec/` is an exact source snapshot at revision `29dac88ecded94457572db3fdfaaaab95fa91525`, imported from the public GitHub mirror and verified byte-for-byte against the admitted Radicle tree (`fc0b6f75a01088faf4f22be57edc073a3fe45e54`). The Radicle RID remains `rad:z2CpqLFpdP36fZXYUK5ZNWxMibpCo`; the snapshot is not a live GitHub fallback.

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
