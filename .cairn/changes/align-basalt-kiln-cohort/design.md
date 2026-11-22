## Context

Lattice's Basalt snapshot `b34d6ac2` and Basalt `main` `5e0f1f4c` have diverged since `4c5c9e9`.

- `main` gained the Kiln authority module, UCAN `fca62a6e` and the ring RS256 OIDC verifier.
- The Lattice snapshot carries three unpublished holder commits that `src/github_ci/exchange.rs` and `src/github_ci/mantle.rs` call:
  - holder-bound OIDC delivery;
  - Events/Result OIDC capabilities;
  - shared canonical Mantle signing.
- A review of that holder delivery found that it did not bind the admitted service and project into the resource the holder signs. Basalt's revised holder series fixes this and makes the exchange one-time per OIDC token.

The GitHub CI session already builds the exact native `basalt::mantle::Request`, with the endpoint owner as subject, and holds the endpoint's `BasaltPolicyRef`. Signing already refuses a grant whose holder is not the endpoint owner.

## Decisions

### Decision: Align to `main` and include the revised holder series

**Choice:** Vendor published Basalt `main` `fac0d7f1`, which is `5e0f1f4c` plus the revised holder series and is on GitHub and Radicle. Gates first ran against the local candidate `6ab48bf`. Against it, `fac0d7f1` changes only Cairn records, CHANGELOG, docs, `flake.nix`, and the wording of three Rust doc comments in `src/oidc_exchange/{core,model,shell}.rs`; no Rust code token changes.

**Rationale:** A verbatim `5e0f1f4c` import deletes APIs that Lattice's GitHub CI exchange and Mantle signing depend on. Re-creating them in Lattice would fork authority logic around Basalt-private issuance. The revised series supplies those APIs and binds holder grants to the exact canonical request.

### Decision: Pass the typed Mantle request and keep Lattice's own checks

**Choice:** `exchange` takes the native Mantle request and the endpoint's `BasaltPolicyRef`, and hands both to `exchange_to_recipient` through `HolderDelivery`.

Before any producer effect, Lattice computes the canonical resource with `mantle::canonical_enforcement`. It requires that resource to sit under `mantle://gateway/v1/` and the policy prefix. It requires the project to be the policy's project and to match the plan's project identity. It requires:

- the stable holder to be the request's subject;
- the request's service to be the plan's;
- the request's ability to be the plan's operation.

After issuance, Lattice still verifies the grant's issuer, holder audience, single capability, exact resource and ability, absent proofs and caveats, and time bounds.

**Rationale:** Basalt enforces the binding at the producer, and Lattice keeps its fail-closed consumer checks, so neither side depends on the other's checks. A failure in the native request or its canonical form now reports `ExchangeError::Policy` from the exchange, where `canonical_resource` reported `Error::Authority`. Both refuse before any token is issued.

### Decision: Exercise the real exchange without the network

**Choice:** `exchange` adapts the production `BasaltHostPorts` into a private `ExchangePorts` whose discovery port is the producer's `DocumentPort` trait. A unit test drives that path with:

- a real RS256 token over Basalt's GitHub claims fixture, signed with a Lattice test-only RSA key through the existing `rustls` ring dev-dependency;
- scripted discovery and JWKS documents;
- Lattice's durable replay store.

**Rationale:** No existing test ran the real producer exchange through Lattice. The public `BasaltHostPorts` keeps its concrete bounded-HTTP port, so production typing does not widen.

### Decision: Sync script only, with full top-level selection

**Choice:** Refresh with `scripts/sync-vendored-sibling.rs`, using the same top-level archive paths as before plus the new `SECURITY.md` and `CHANGELOG.md`. Do not hand-edit any vendored file.

**Rationale:** This keeps `SOURCE` an exact description of the import and matches the vendor README's refresh rule.

### Decision: Lockfile follows the vendored manifests

**Choice:** Let Cargo re-resolve only what the vendored Basalt and UCAN manifests require. This means the k256 0.14 family, and `ring` replacing `jsonwebtoken` as a normal dependency. No other dependency is updated.

**Rationale:** This keeps the dependency delta reviewable and traceable to the producer cohort.

### Decision: Octet baseline delta comes from the integration re-key

**Choice:** This change does not edit `octet-baseline.json`. The cohort leaves 0 new findings and 2 stale ones, both at `src/http/native.rs:8` (`use zeroize::Zeroize as _`):

- `non_trait_imports` `b3:bd2adf4b497013d0`;
- `renamed_imports` `b3:cd08f981f3e56a47`.

They go stale because `zeroize_derive` leaves the graph along with `jsonwebtoken`, `p256` and `p384`, so `zeroize::Zeroize` now resolves only to the trait. This was measured on the earlier candidate, before any Lattice source changed. The holder adaptation in `src/github_ci/exchange.rs` may add findings on its new code. The relocation integration's single Octet re-key regenerates the baseline with `cargo-octet check --write-baseline` and reviews any finding that is semantically new. The expected baseline delta from this change is these two stale removals, plus whatever that re-key reports for the adapted exchange.

**Rationale:** A separate prune here would conflict in `octet-baseline.json` with the integration's single re-key. The baseline is never hand-edited.

### Non-claim: Mantle snapshot's own Basalt pin

The vendored Mantle snapshot still declares Basalt `4c5c9e919b4573a7865559db2dc98a99c0c58144` in its own manifest and lock. Lattice builds only Mantle sub-crates that don't depend on Basalt, not Mantle's root crate, so that pin is outside Lattice's dependency graph. This change makes no claim about Mantle's Basalt cohort.

## Risks / Trade-offs

- The k256 0.14 RustCrypto family enters the lock at the same versions as Basalt's own lock: elliptic-curve 0.14, ecdsa 0.17, crypto-bigint 0.7, sec1 0.8, hybrid-array 0.4, digest 0.11 and signature 3. The 0.10/2.x digest and signature lines stay alongside them for ed25519 and rsa. Lattice code does not use these crates directly.

## Known Limitation: Bounded Exec stdin EPIPE race

Vendored Bounded Exec stays at the Radicle-admitted `29dac88ecded94457572db3fdfaaaab95fa91525`. That tree is byte-identical to the admitted Radicle tree and is pinned by the admission and cutover evidence.

Under load, its stdin EPIPE race can report a successful process node as a capture failure (`AdapterRejected`). Bounded Exec `c33f1bd` fixes this: unread stdin never fails a run. That revision exists only on GitHub, and Bounded Exec's GitHub, Radicle (`1baa4f5`) and local histories have diverged. The issue is tracked at https://github.com/OnixResearch/bounded-exec/issues/2.

Adoption waits for a reviewed fixed revision on the admitted Radicle RID `rad:z2CpqLFpdP36fZXYUK5ZNWxMibpCo`, with regenerated admission and cutover evidence. Taking it from GitHub would break `lattice.repository_validation.radicle_bounded_exec_source`. Tests that drain stdin with `cat > /dev/null` keep their workaround until then.
