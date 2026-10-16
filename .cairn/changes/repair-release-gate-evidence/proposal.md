## Why

Kiln's CI runtime smoke is passing, but the current Lattice checkout still fails accepted-spec shape validation, Octet stable-ID comparison, and traceability coverage. The reported coverage has 169 missing IDs and two dangling markers; some implementation markers live in maintained build files outside the admitted profile. These are release-gate failures, not permission to waive requirements or invent evidence.

## What Changes

- Canonicalize accepted requirement/scenario marker placement without changing requirement IDs or weakening normative behavior.
- Bind uncovered requirements only to inspected implementation and behavioral checks that actually establish their bounded claims; complete uncertain regression coverage rather than marking an unsupported claim.
- Admit exact repository-owned build-manifest and Nix policy files as tool evidence, without scanning vendor roots or broadening accepted requirement sources.
- Replace incidental source-fragment migration tests with actual bounded migration behavior and migrate Octet IDs only when the complete warning multiset is preserved.
- Regenerate the current missing-set inventory and record final repository gates and the Kiln consumer observation.

## Impact

- **Files**: accepted specifications, assertion-local source/test markers, repository-validation helpers, `cairn-policy/default.ncl` and generated policy/inventory, reviewed `octet-baseline.json`, and maintained verification documentation.
- **Testing**: existing spec-shape, source-discovery, migration, and affected consumer-visible regressions; `scripts/check.rs`, `octet-precommit-check`, Cairn validation/gates/coverage and inventory, full x86_64 Nix checks, and the real local Kiln/Lattice CI path.
- **Consumer and owner**: Kiln's GitHub CI adapter needs a checked Lattice producer. Lattice maintainers own producer gates and evidence; Kiln maintainers own CI integration and deployment receipts.
- **Authority**: this change does not authorize repository governance mutation, signing, source publication, access-policy changes, or use of synthetic fixtures as production credentials. Any live GitHub round-trip must use operator-provisioned exact deployment inputs.
