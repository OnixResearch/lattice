## Context

The OIDC exchange runtime closure verifies RS256 with `ring` 0.17.14. The source contract
binds `ring_version` and `ring_checksum`, and the exported source boundary names them under
`rs256_verifier`. `jsonwebtoken` 11.0.0 remains only in `[dev-dependencies]`, where the
fixture tests use it to mint tokens.

## Decisions

### Decision: Assert the ring pin at every layer the contract names

**Choice:** The check asserts the same version and checksum in four places:

- the runtime `[dependencies]` line;
- the Nickel contract identity;
- the exported boundary JSON;
- the exact `Cargo.lock` package entry, matched as name, version, crates.io source and
  checksum on consecutive lines.

**Rationale:** Matching the checksum anywhere in the lock could pass on another
package's entry. A consecutive-line match binds it to `ring` 0.17.14 from crates.io.

### Decision: Keep jsonwebtoken assertions, scoped to dev-dependencies

**Choice:** Keep the exact `jsonwebtoken` manifest line and lock checksum. Match the manifest
line only in `[dev-dependencies]`, and reject any `jsonwebtoken` entry in `[dependencies]`.

**Rationale:** It is still a pinned dependency of the test build, so its exact pin stays
asserted. The ring migration's purpose was to keep it out of every consumer's runtime
closure.

## Risks / Trade-offs

- The sections are extracted with `awk` on TOML section headers. A dependency declared
  through a `[target.*.dependencies]` table would not be seen. Basalt declares none.
