# Validation

## Implementation evidence

- `NIX_CONFIG='secret-key-files =' nix develop -c env CARGO_TARGET_DIR=/tmp/basalt-authority-tests RUSTC_WRAPPER= CARGO_NET_GIT_FETCH_WITH_CLI=true cargo test --locked --test authority_core --no-default-features`
  - PASS: 8 authority-core tests passed.
  - Log BLAKE3: `c7695051fa7627b1622a29c58d8b495c46de3c5df30c9200daff86665d36a054` (`/tmp/basalt-authority-focused.log`).
- `NIX_CONFIG='secret-key-files =' nix develop -c env CARGO_TARGET_DIR=/tmp/basalt-authority-clippy RUSTC_WRAPPER= CARGO_NET_GIT_FETCH_WITH_CLI=true cargo clippy --locked --all-targets --no-default-features -- -D warnings`
  - PASS.
  - Log BLAKE3: `a5bbbb169eb65f55cf380954811dc95ba89850373d7c370ecf090134cb00c27a`.
- Direct release verifier:
  - `NIX_CONFIG='secret-key-files =' nix develop -c rustc tools/release-verify.rs -o /tmp/basalt-authority-release-verify-final`
  - `NIX_CONFIG='secret-key-files =' nix develop -c env RELEASE_VERIFY_RECEIPT_DIR=/tmp/basalt-authority-release-receipts-final /tmp/basalt-authority-release-verify-final`
  - PASS: release verifier ran policy freshness, authority-core and UCAN-semantics fixture freshness, formatting, license metadata, no-default and CLI-feature nextest rails, focused property/Steel/UCAN/semantics/authority tests, doctests, Clippy, package validation, package contents, Clankers consumer fixture deferral, and docs.
  - Direct release log BLAKE3: `40afd2e04c31efcca78b72c95e0a31863ea2c9ceafd22e2eea0ede74540d46e8`.
  - Summary receipt BLAKE3: `6bb8b1055e1e40a495772b6eee21293dfb0af709ab3b5f167c419f056a06c35c`.
  - Authority receipt BLAKE3: `3bba5536562b5c4071c768fe703888d0000abc8ce6b2983ef17c443c184a62fc`.
- `nix flake check path:$PWD -L --option secret-key-files ''`
  - PASS.
  - Log BLAKE3: `ec379f6513c322aff488fc5db3c6b004227ac654a370e60dc474c817c50a636b`.

## Cairn evidence

Commands used the canonical Cairn policy at `../cairn/cairn-policy/generated/cairn-policy.json`.

- `nix run path:../cairn#cairn -- gate proposal ucan-basalt-authority-core --root . --policy ../cairn/cairn-policy/generated/cairn-policy.json`
  - PASS after the change spec was converted to explicit `## ADDED Requirements` delta form.
  - Receipt hash: `4a6d596b5b0601cee744c5e3dbe31bbde6bd0566208af9ec445de828f4cccbc1`.
  - Log BLAKE3: `f0497b1406999c22bcafa09f26c24d6aec2ea43994ecb292af026924f25278f2`.
- `nix run path:../cairn#cairn -- gate design ucan-basalt-authority-core --root . --policy ../cairn/cairn-policy/generated/cairn-policy.json`
  - PASS after the change spec was converted to explicit delta form.
  - Receipt hash: `77975e14d851b171c2dca0c9107380f9723154a70c9d0ee80b6ed551e44bbaab`.
  - Log BLAKE3: `83a7ed25690cfc34912a6e83a00c0f6ffed85c2e70f307e27a1cd764f7722f32`.
- `nix run path:../cairn#cairn -- gate tasks ucan-basalt-authority-core --root . --policy ../cairn/cairn-policy/generated/cairn-policy.json`
  - PASS after all tasks were marked complete and the change spec was converted to explicit delta form.
  - Receipt hash: `7c694b06922fab7f565e83bb9df7b3e9715dbc209791226ca020da9a699c85c1`.
  - Log BLAKE3: `c4477208625c79be0abc0348f6789068efc7c1f1390ee148503acc05b19aaad4`.
- `nix run path:../cairn#cairn -- sync ucan-basalt-authority-core --root . --policy ../cairn/cairn-policy/generated/cairn-policy.json`
  - DRY-RUN BLOCKED only by unrelated accepted legacy spec `./cairn/specs/test-hardening/spec.md`; merge preflight recognized six added `ucan-token-flow` requirements.
  - Log BLAKE3: `1b2e5ffb548941362d67fe936a6f33d7184b2c2111e060de2d0392b2e3a5318a`.
- `nix run git+file:///home/brittonr/git/OnixResearch/cairn?rev=8774a1f8fa2bea8c8e7bc6741ae31d56563c8802#cairn -- sync ucan-basalt-authority-core --root . --policy ../cairn/cairn-policy/generated/cairn-policy.json --execute`
  - PASS: compatibility Cairn revision executed the bounded accepted-spec sync.
  - Receipt hash: `d72f30437c120c4e6e72fd0ae6df4e64a2007c22917fe5b23dafceb644f3a65a`.
  - Log BLAKE3: `e10d9b5376a58ae04438c50466b645fd81726446466ee99fc7d8f58a150f741a`.
- `CAIRN_ARCHIVE_DATE=2026-07-15 nix run git+file:///home/brittonr/git/OnixResearch/cairn?rev=8774a1f8fa2bea8c8e7bc6741ae31d56563c8802#cairn -- archive ucan-basalt-authority-core --root . --policy ../cairn/cairn-policy/generated/cairn-policy.json --execute`
  - PASS: moved the completed change to `cairn/archive/2026-07-15-ucan-basalt-authority-core/`.
  - Receipt hash: `521563e6d4f36da6180aa25c89db14e8d372d486cd6c390792dc13ede8b9c2e0`.
  - Log BLAKE3: `5e947563d37ab978cd752aa891d6e36efd4d04e53e5b3fba7dd428243be4de6e`.
- `nix run git+file:///home/brittonr/git/OnixResearch/cairn?rev=8774a1f8fa2bea8c8e7bc6741ae31d56563c8802#cairn -- validate --root . --policy ../cairn/cairn-policy/generated/cairn-policy.json`
  - PASS after archive.
  - Log BLAKE3: `cc845300f4cb6643d3c649f310046e0da36e20a6bb0927dd3ae497935e0253e4`.
- `nix run path:../cairn#cairn -- validate --root . --policy ../cairn/cairn-policy/generated/cairn-policy.json`
  - CURRENT-SCHEMA BLOCKED by unrelated accepted legacy spec: `./cairn/specs/test-hardening/spec.md: lifecycle_validation.spec_substance.zero_requirements`.
  - The archived authority-core change is no longer active; current validation reports no change issues for it.
  - Log BLAKE3: `4217f90660e048002cf198ff879a8cf9ff46c510c56c4d6dc079c621fee023ee`.

## Review note

A bounded adversarial review was run against the authority boundary. Non-applicable findings alleged runtime token parsing/Nickel evaluation that this implementation does not contain. Actionable edge-case coverage was added for malformed verified grant refs, omitted receipt grant bindings, rejected receipt refs, multiple policy refs, and missing JSON grant refs.
