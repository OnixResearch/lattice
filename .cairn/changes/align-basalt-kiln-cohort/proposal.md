## Why

Lattice vendors Basalt `b34d6ac27ad396530fb799e79223acf71112ee0f`. That snapshot has no Kiln authority module, so Lattice cannot later verify Kiln-issued grants with the same Basalt and UCAN code that Kiln uses. Basalt `main` `5e0f1f4cad925a50f7dd5cdbcae05a04e7b92463` adds:

- Kiln implementation v2 framing, source v1 operations and an explicit delegate rule;
- vendored UCAN `fca62a6e`, with k256 0.14;
- the ring-based RS256 OIDC verifier.

This change is the L2 cohort alignment, a prerequisite for that later verification.

`b34d6ac2` is not an ancestor of `5e0f1f4c`. It carries three unpublished holder commits that Lattice's GitHub CI path calls: OIDC holder delivery, the Events/Result OIDC capabilities and `mantle::canonical_enforcement`. A security review of that series found that holder-delivered grants did not bind the admitted service and project into the signed resource. Basalt's holder series therefore now:

- takes the typed Mantle request and policy reference;
- requires the request's subject to be the recipient;
- requires its service and project to be the policy's;
- grants exactly the request's canonical resource;
- allows one exchange per OIDC token.

## What Changes

- Re-vendor `vendor-cargo-deps/basalt` through `scripts/sync-vendored-sibling.rs` at published Basalt `main` `fac0d7f1fd7ac936ca4d3f251f3287666e93d4bb`, which is `5e0f1f4c` plus the holder series and is available on GitHub and Radicle. Gates first ran against the local candidate `6ab48bffa29085f875d307dfbed1b73c9e5d1d6f`; `fac0d7f1` differs from it only in non-Rust files and Rust doc comments. Record the revision in `SOURCE` and the vendor README. The archive path selection gains the new top-level `SECURITY.md` and `CHANGELOG.md`.
- `github_ci::exchange::exchange` passes the typed native Mantle request that the GitHub CI session already builds, plus the endpoint's `BasaltPolicyRef`, to Basalt's holder delivery. It no longer passes a pre-rendered resource string.
- Lattice keeps its own checks:
  - before any producer effect: exact project, service, operation, the canonical resource prefix, and that the stable holder is the request's subject;
  - after issuance: cryptographic verification of the issued grant.
- Update `Cargo.lock` only where the vendored manifests require it.
- Add a consumer test that runs the real Basalt holder exchange through Lattice's exchange path: a real RS256 OIDC token, scripted discovery and JWKS documents, and Lattice's durable replay store.

## Out of Scope

- Verifying Kiln-issued grants in Lattice; a later change consumes this cohort for that.
- Changing the separately vendored Mantle snapshot's own Basalt pin.
- Publishing the holder series on Basalt `main`; its owner published it as `fac0d7f1`.
- Adopting Bounded Exec `c33f1bd`. The stdin EPIPE race it fixes remains a known limitation (see design).

## Impact

- **Files**:
  - `vendor-cargo-deps/basalt/**` (sync-script output only);
  - `vendor-cargo-deps/README.md`;
  - `Cargo.lock`;
  - `CHANGELOG.md`;
  - `src/github_ci/{exchange,session,mantle}.rs`;
  - `tests/fixtures/github-ci/oidc-rs256-test-{key.pem,jwks.json}`.
- **API**: `github_ci::exchange::exchange` takes `(&basalt::mantle::Request, &BasaltPolicyRef)` instead of `(&str project, &str resource)`.
- **Testing**:
  - focused tests for every Lattice module that uses Basalt;
  - `cargo test -p lattice --lib` filtered to authority, OIDC and GitHub CI;
  - `cargo clippy -p lattice --no-deps --all-targets -D warnings`;
  - Cairn validation.
  - The Octet re-key happens in the relocation integration.
