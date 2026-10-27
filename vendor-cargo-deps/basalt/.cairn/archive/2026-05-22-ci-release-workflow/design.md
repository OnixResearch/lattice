# Add CI release workflow Design

## 1. CI exposes the existing verifier instead of inventing a second rail

**Choice:** Add a workflow that runs the repository's existing Nix release check rather than duplicating Cargo commands inline.

**Rationale:** `tools/release-verify.rs` and `checks.release` are already the maintained release rail. Repeating fmt/test/clippy/package commands in CI would create drift and weaken local/CI parity.

## 2. Public/sandbox CI and full private-dependency verification are separate modes

**Choice:** Make `nix flake check -L` the default workflow job and document that the full direct verifier requires private Git dependency access.

**Rationale:** The crate currently depends on a private UCAN Git source. The flake check is intentionally sandbox-aware and records Cargo-dependent steps as deferred. Hosted CI without configured SSH credentials can still prove Nix wiring, policy freshness, workflow syntax, and verifier receipt generation without pretending it ran private-network Cargo rails.

## 3. No credentials in repository content

**Choice:** The workflow and docs may name required secret classes or SSH setup points, but must not include private keys, tokens, deploy keys, or credential values.

**Rationale:** CI support should reduce release risk without creating a credential-leak path. Secret provisioning is an operator action outside this repository change.

## 4. Evidence remains inspectable

**Choice:** The workflow should preserve or upload release-verifier receipts when practical, and README text should explain what the default CI job proves versus what the direct verifier proves.

**Rationale:** The release verifier already emits JSON/Markdown receipts. CI should make those artifacts reviewable instead of only showing transient logs.
