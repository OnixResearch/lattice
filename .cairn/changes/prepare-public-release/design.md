## Context

The default Lattice workspace mixed a private Tile compose binding with the
public runtime. Cargo locks optional dependencies even when their features are
off, so feature-gating alone would not produce an anonymously acquirable Nix
vendor closure. The public network service does not need the compose binding.

## Decisions

### Decision: Keep private Tile source external

**Choice:** Move the Tile observation shell and example into `compose/`, give
that directory its own workspace and lockfile, and exclude it from the root
workspace. The root package no longer depends on `tile-sdk`, `tile-protocol`, or
`lattice-compose-binding`.

**Rationale:** Vendoring or publishing Tile source would disclose private source
without explicit authorization. Feature-gating alone would still make
`cargo vendor` and the Nix derivation fetch the optional Git dependency. A
separate workspace preserves the existing implementation for operators with
authorized Tile access and removes it from the public closure.

**Trade-off:** `lattice::compose_surface` is no longer a library export of the
default package. The compose workspace owns that surface and its tests.

### Decision: Preserve exact revisions, change only acquisition routes

**Choice:** Keep the locked revisions for Valence, Artifact, Trellis, Cairn, and
Octet. Switch direct Valence to HTTPS, patch the transitive SSH-only Artifact and
Trellis sources to their exact HTTPS revisions, and override the Cairn and Octet
flake inputs to HTTPS.

**Rationale:** A public release must build without an SSH key or GitHub write
access. Changing only the source route preserves dependency content and avoids a
broad dependency update. The patches are explicit and removable when the
upstream manifests switch to HTTPS.

### Decision: Release the service, not a hosted product

**Choice:** Deliver a Nix-built binary archive, a local systemd unit, and
operator documentation. Do not include a hosted service, remote authentication,
TLS termination, multi-tenant isolation, or a support SLA.

**Rationale:** The implemented service is a local Unix-socket workflow exchange
with peer user-ID admission. Describing it as an internet-facing or multi-tenant
service would exceed the checked boundary.

### Decision: Ground launch material in checked capabilities

**Choice:** GTM material links exact capabilities and non-claims instead of
asserting correctness, security, or production fitness. Private Tile
distribution remains an open decision.

**Rationale:** Release evidence is scoped to local checks and gates. Marketing
claims must not outrun the repository's evidence boundary.

## Risks / Trade-offs

- The compose workspace has its own lockfile and must be refreshed with Tile
  access; it is not covered by the public CI rail.
- The HTTPS patches add a temporary divergence between Lattice's manifest and
  upstream manifests. Remove them when upstream declares HTTPS directly.
- The release workflow requires a GitHub tag and repository write permission; it
  does not publish to a package registry or hosted service.
- The default binary no longer exports the compose surface. Consumers requiring
  it must build the separate workspace.
