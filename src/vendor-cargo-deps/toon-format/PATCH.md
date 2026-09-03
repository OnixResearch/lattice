# Lattice TOON vendor patch

This directory contains the MIT-licensed `toon-format` crate release `0.5.0` from revision `2136cb1a35f3ed63be733bb74d36b89d3b4592dd`.

`UPSTREAM-FILES.b3` covers the selected upstream library source, license, README, changelog, original manifest, and revision metadata. The import omits package-only CI, examples, tests, docs, and the nested lockfile. Run `b3sum --check UPSTREAM-FILES.b3` from this directory to verify those bytes.

Lattice keeps two trees. This directory is the byte-exact pristine record. The sibling `toon-format-compile/` tree is derived for in-crate compilation by `scripts/sync_toon_format.rs`: each file's tail `#[cfg(test)] mod tests` block is dropped, and each documented example's opening bare code fence becomes `ignore`. The upstream tests and doctests assume `serde_json/preserve_order` key ordering and the external `toon_format` crate name, neither of which Lattice's featureless in-crate embedding provides. The derived tree's manifest identity is pinned in the sync script and the Nix policy check. Lattice's own strict round-trip corpus and route tests cover the codec behavior on this build.

The generated Cargo manifest is also replaced. The replacement keeps the library dependencies and declares disabled compatibility feature names. It removes the binary, examples, tests, development dependencies, and optional CLI or TUI dependencies. It also removes `serde_json/preserve_order` so the accepted Lattice prompt bytes, JSON key ordering, and Octet identities stay unchanged.

No upstream Rust source statement is reordered or rewritten; the only derivations are dropping each file's trailing upstream test module and marking documented example fences `ignore`. The patch adds no executable, filesystem, network, prompt, model, or workflow authority.

Use `scripts/sync_toon_format.rs SOURCE DESTINATION` to reproduce both trees from an extracted upstream `toon-format-0.5.0` crate directory.
