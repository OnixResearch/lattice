# Basalt

Basalt enforces UCAN contract boundaries across Nickel policy artifacts and Steel DTO backends. It is a standalone Rust and Nix repository.

## Flow

1. Author contract policy in `contracts/policy.ncl`.
2. Export the policy with Nickel.
3. Load the generated JSON in Rust.
4. Make sure that the selected contract permits the resource and ability.
5. Make sure that supplied UCAN capabilities authorize the same resource and ability.

Export the policy with this command:

```bash
nickel export contracts/policy.ncl > contracts/generated/policy.json
```

The repository uses a small functional core and imperative shell. Nickel owns policy structure and reviewed defaults. Rust owns runtime enforcement and receipts.

## Publication status

Basalt is under active development. Its UCAN dependency uses a checked-in reviewed snapshot.

The snapshot records integrated UCAN commit `277ab768eb532ac0c715a3bbff2a7dc37a702f11`.
That revision is published on the UCAN `main` branch. The repository-owned snapshot remains the durable build boundary.

`Cargo.toml` contains package metadata for downstream discovery. The crate retains the `AGPL-3.0-or-later` license.

Basalt was previously named `ucan-nickel-contracts`. That name can remain in historical Cairn archives and stable policy schema identifiers.

## License boundary

`Cargo.toml` declares `AGPL-3.0-or-later`. [`LICENSE`](LICENSE) contains the complete license text.

Downstream applications must review the license before adoption. Third-party dependencies retain their own terms.

Earlier grants remain valid. This README does not grant an alternate license, a compatibility exception, or legal advice.

## Quick start

```bash
mkdir -p contracts/generated
nickel export contracts/policy.ncl > contracts/generated/policy.json
cargo test
cargo run --features cli -- enforce \
  --policy contracts/generated/policy.json \
  --contract document-write \
  --resource did:key:z6Mkresource/docs/report \
  --ability document/write \
  --capability did:key:z6Mkresource/docs/report=document/write
```

The sample request produces this output:

```json
{"allowed":true,"reason":"allowed"}
```

## Policy artifacts

The crate includes these policy artifacts:

- reviewed Nickel source at `contracts/policy.ncl`
- generated JSON at `contracts/generated/policy.json`

The Rust library exposes the generated artifact through `DEFAULT_POLICY_JSON` and `default_policy()`.

A downstream application can use the packaged policy for a smoke test. A production application must select one of these options:

- supply reviewed Nickel-exported JSON to `parse_policy_json`
- use `default_policy()` when the packaged contracts match the application

`Cargo.toml` includes the generated artifact in the package. The release rail checks source and generated artifact freshness.

Run the focused freshness check:

```bash
nix build .#checks.$(nix eval --raw --impure --expr builtins.currentSystem).policy-export --no-link
```

## Library API

Applications can use the enforcement API with default features:

```rust
use basalt::{CapabilityGrant, EnforcementRequest, default_policy, enforce};

let policy = default_policy()?;
let request = EnforcementRequest::new(
    "document-write",
    "did:key:z6Mkresource/docs/report",
    "document/write",
)
.with_capability(CapabilityGrant::new(
    "did:key:z6Mkresource/docs/report",
    "document/write",
));

let receipt = enforce(&policy, &request)?;
assert!(receipt.is_allowed());
assert_eq!(receipt.reason(), "allowed");
# Ok::<(), Box<dyn std::error::Error>>(())
```

## CLI feature

Default library use does not include CLI parser dependencies. Enable the `cli` feature to use the `basalt` command:

```bash
cargo run --features cli -- enforce --help
```

## UCAN dependency source

The crate uses `vendor/ucan/` as its reviewed UCAN source boundary.
`vendor/ucan-revision.txt` records the full source revision.
`vendor/ucan.snapshot` records deterministic BLAKE3 identities for every imported file.

Run the offline check with these commands:

```bash
nix develop -c rustc --edition=2024 tools/sync-vendored-ucan.rs -o /tmp/basalt-sync-vendored-ucan
nix develop -c /tmp/basalt-sync-vendored-ucan --check
```

Use an explicit local UCAN repository only for source parity or a reviewed refresh:

```bash
nix develop -c /tmp/basalt-sync-vendored-ucan --check-source-revision \
  /path/to/ucan FULL_GIT_OBJECT_ID
tools/sync-vendored-ucan.rs --import-revision \
  /path/to/ucan FULL_GIT_OBJECT_ID
```

The importer rejects branch names, abbreviated object IDs, uppercase object IDs, and unavailable commits before snapshot replacement.

Read [`docs/vendored-ucan.md`](docs/vendored-ucan.md) for the imported paths, license boundary, and refresh rules.

## UCAN and Basalt authority core

`basalt::authority_core` is a pure deterministic decision boundary. It accepts loaded policy values and references to already-verified UCAN grants and receipts.

The core checks exact bindings for these values:

- request and policy
- grant and receipt
- caveat and replay decision
- non-claims

Then it routes one selected grant through the existing `enforce` core.

`adapt_enforcement_request` preserves existing public request fields. It requires one explicit verified reference for each capability.

The core does not perform these functions:

- parse tokens or check signatures
- resolve keys
- read clocks
- load proofs
- consult revocation or replay stores
- evaluate Nickel
- treat token possession as current authority

An allow applies only to the exact contract, resource, ability, and request binding.

It does not grant transport, storage, persistence, replay-store, key-resolution, business, release, or whole-system authority.

Read [`docs/authority-core.md`](docs/authority-core.md) for the full boundary.

## Nostr identity bindings

`basalt::nostr_identity` maps one reviewed opaque `VerifiedNostrCredential` to one stable subject in typed Nickel policy.

The pure core validates exact key, request, audience, purpose, policy, lifecycle, digest-role, and non-claim linkage. Active keys can authenticate. Retired and revoked keys fail closed.

Authentication never grants a resource or ability. Optional composition requires a separate exact authorization result for the same subject, request, and policy.

Authentication and composition receipts are redacted, BLAKE3-bound, and `recorded_only`. The external Nostr event ID keeps its NIP-01 SHA-256 role.

Read [`docs/nostr-identity-bindings.md`](docs/nostr-identity-bindings.md) for policy, rotation, privacy, receipt, and authority boundaries.

## Raw UCAN enforcement

`basalt::ucan_enforcement` verifies one raw holder-signed invocation for one exact normalized request.

The caller injects key, proof, revocation, caveat, replay, time, and limit boundaries. The request cannot contain caller-supplied capability grants.

Keep bearer text in `CompactToken`. Pass only borrowed `as_str()` views, and never copy bearer text into arguments, logs, or serde values.

A typed action maps to one contract, ability, length-prefixed BLAKE3 operation profile, and exact request digest.

Basalt builds its authority decision and redacted receipt before fresh holder verification. UCAN then calls replay once as the final consuming commit.

A policy denial makes no replay call. After replay succeeds, Basalt only moves prebuilt values into the private live capability.

`LiveUcanEnforcement` borrows the exact request. It has no public constructor, decision getter, clone implementation, or serde implementation.

Read [`docs/ucan-enforcement.md`](docs/ucan-enforcement.md) for API use, ordering, reasons, receipts, and bounded claims.

## UCAN semantics compatibility

Typed fixtures in `contracts/ucan-semantics/` bind these identities:

- the exact runtime UCAN revision
- the reviewed UCAN support-catalog revision and BLAKE3 identity
- the generated Basalt policy identity

`basalt::ucan_semantics` accepts caller-supplied decisions for delegation, caveats, proof traversal, and replay.

It fails closed for each fact that is not admitted. It sends only admitted direct and delegated grants to `enforce`.

This boundary does not reimplement UCAN token semantics in Nickel or Rust. It does not prove these properties:

- current revocation data
- correct key resolution
- UCAN cryptographic correctness
- durable replay storage
- complete application authorization

Read [`docs/ucan-semantics-contract.md`](docs/ucan-semantics-contract.md) for identities, fixtures, flow, and commands.

## Release checks

Use a fresh target directory and clear inherited Rust compiler wrappers. This avoids stale shared Cargo artifacts.

Cargo resolves UCAN from the reviewed `vendor/ucan/` snapshot. The build does not require a remote UCAN revision.

Basalt is a private crate with `publish = false`.
Do not create a registry tarball. Cargo would replace the path with the unrelated crates.io package `ucan ^0.1.0`.
The release rail validates the private source inventory and package list without creating that unsafe tarball.

Before a tag or downstream handoff, run the full release rail:

```bash
nix develop -c rustc --edition=2024 tools/release-verify.rs -o /tmp/basalt-release-verify
nix develop -c /tmp/basalt-release-verify
```

The default flake check exposes the same verifier:

```bash
nix flake check -L
```

The sandboxed release check validates repository inputs without running Cargo. Its receipt marks Cargo-dependent checks as deferred.

Run the direct verifier in a trusted environment for full Cargo coverage.

The GitHub Actions workflow uses the sandbox-safe flake check without repository secrets.

### Release rail coverage

The direct rail performs these checks:

- the vendored UCAN revision, file manifest, source modes, and preserved license files
- policy, authority-core, and UCAN-semantics fixture freshness
- `cargo fmt --check`
- AGPL license metadata
- no-default and CLI-feature tests through `cargo nextest`
- deterministic authority, property, Steel-adapter, token, and semantics fixtures
- doctests and Clippy for both feature sets
- private source inventory and package-list contents without a registry tarball
- the `../clankers/` consumer fixture when present
- library documentation without dependencies

The rail uses fresh `/tmp/basalt-release-*` target directories. It clears `RUSTC_WRAPPER` and enables Git CLI fetching.

Direct runs write these receipts:

- `target/release-verify/summary.json`
- `target/release-verify/summary.md`
- `target/release-verify/authority-core.json`
- `target/release-verify/ucan-semantics.json`

Nix checks place these receipts in the check output. CI uploads summary files when the build result provides them.

### Focused checks

Use these commands during development:

```bash
nix build .#checks.$(nix eval --raw --impure --expr builtins.currentSystem).policy-export --no-link
nix build .#checks.$(nix eval --raw --impure --expr builtins.currentSystem).authority-core-fixtures --no-link
nix build .#checks.$(nix eval --raw --impure --expr builtins.currentSystem).ucan-semantics-fixtures --no-link
nix develop -c env CARGO_TARGET_DIR=/tmp/basalt-target RUSTC_WRAPPER= cargo nextest run --no-default-features
nix develop -c env CARGO_TARGET_DIR=/tmp/basalt-target RUSTC_WRAPPER= cargo nextest run --features cli
nix develop -c env CARGO_TARGET_DIR=/tmp/basalt-target RUSTC_WRAPPER= cargo test --doc --no-default-features
# The feature-enabled test rail includes the external-style CLI smoke and negative CLI tests in tests/cli_feature.rs.
```

## Downstream compatibility

Run focused consumer checks with this command:

```bash
nix develop -c rustc --edition=2024 tools/downstream-compat.rs -o /tmp/basalt-downstream-compat
/tmp/basalt-downstream-compat --workspace-root .. --consumer clankers --consumer aspen --consumer onixos --require-present
```

The runner creates temporary worktrees in `target/downstream-compat/`. It links the current Basalt checkout into each consumer.

Then it runs narrow checks for Clankers, Aspen, and OnixOS. The old `--consumer onix-modules` spelling remains an alias for `onixos`.

The runner is local-only. Keep sibling checkouts under the same workspace root. Do not require GitHub Actions tokens for this matrix.

## References

- [UCAN](https://github.com/OnixResearch/ucan) — source project for the reviewed vendored authority and holder-verification boundary.
- [Nostr protocol and NIPs](https://github.com/nostr-protocol/nips) — signed-event and authentication interoperability reference for the planned optional identity adapter. Basalt retains stable subject policy and UCAN authority.
