# Persist versioned operation data validation

Date: 2026-08-15

## Authoritative gates

- `env CARGO_TARGET_DIR=target/operation-data ./scripts/check.rs`: passed after refreshing only the reviewed README and `src/store/mod.rs` traceability hashes changed by this work.
- `nix flake check -L --option secret-key-files '' --builders ''`: passed with local builders. The check finished with `all checks passed!`.
- `nix develop -c octet-precommit-check`: passed with status `warning-only`, zero error findings, zero new findings, and zero stale findings. The reviewed baseline contains 855 accepted and 855 current identities.

## Focused implementation checks

- `cargo clippy --no-deps --lib --bins --test test_operation_data --test test_operation_contracts --test test_operation_cli -- -D warnings`: passed.
- `cargo test --test test_operation_data`: 17 passed, including positive and negative coverage for canonical resources, opaque files, exact and latest queries, sensitive authority, bounded candidates, retention, orphan separation, projections, deletion failures, crash recovery, mutation ownership, and concurrent writers.
- `cargo test --test test_operation_contracts`: 16 passed, including exact operation-data invocation binding.
- `cargo test --test test_operation_cli`: 5 passed, including inert previews, confirmed deletion, query selection, metadata-only projection, and create-only publication.
- `nix run nixpkgs#nickel -- typecheck policy/operation-data.ncl`: passed.
- Valid Nickel selector export: passed.
- Invalid identity selector export: failed as required.
- `cargo test --test test_octet_baseline_migration`: passed both positive and negative migration guard tests.

## Bounded claims

The implementation proves deterministic admission, identity, persistence, query, binding, retention, recovery, and metadata-projection facts for admitted local inputs. It does not prove external truth, secure erasure, secret discovery, payload confidentiality outside the admitted authority boundary, or execution authority from recorded data.

The existing broad Tracey advisory debt remains outside this change. This change did not promote unrelated missing or dangling identities.
