## Context

The root runtime already declares AGPL-3.0-or-later, while the workspace default and reusable consumer packages still declare MIT. The owned operation-pack fixture also advertises MIT. Imported source and exact old Git revisions are distinct ownership boundaries.

## Decisions

### Decision: One prospective owned-source grant

**Choice:** Use AGPL-3.0-or-later for all new owned contributions, retain the full AGPL text at the repository root and standalone contract package roots, and scope retained MIT texts explicitly to historical grants. Preserve both existing copyright attributions.

**Rationale:** Metadata, package-local grant texts, and owned fixture declarations must agree. A prior MIT grant is irrevocable, including when the previously granted content remains byte-identical; changing the declaration cannot erase that grant.

### Decision: Preserve immutable identities and acquisition boundaries

**Choice:** Leave imported snapshots, historical receipts, reviewed old-source fixtures, dependency license assertions, credential URLs, source pins, lockfiles and generated runtime plans unchanged. Update only current owned pack authoring and consumers; its Nickel manifest is authoritative and the example must emit matching license fields.

**Rationale:** Source preparation does not publish a revision, refresh imports, authorize effects, or establish anonymous dependency acquisition.

### Decision: Isolate inherited Rats metadata outside its snapshot

**Choice:** Add an import-only wrapper workspace at `vendor-cargo-deps/rats/Cargo.toml`, outside `rats/source`, and exclude that workspace from Lattice's owned workspace. Its MIT import default and inherited dependency/edition values preserve the existing sparse Rats manifests without editing imported bytes. The wrapper authoring itself is owned AGPL source; its MIT field describes imported packages only.

**Rationale:** Sparse Rats packages currently inherit Lattice's MIT workspace default. An AGPL owned default without isolation would falsely relabel these historical imported packages. No source revision or dependency version changes are needed.

The existing Rats importer embeds the checked wrapper and preserves or reproduces it beside each refreshed source snapshot, refusing to overwrite a different workspace. The wrapper is the single owned metadata source, not a duplicated imported tree.

## Risks / Trade-offs

- Consumers of newly changed contract source must review the prospective AGPL boundary separately from Kiln's retained MIT snapshot. Current private/SSH dependency acquisition prerequisites remain; final verification is not asserted during this preparation.
