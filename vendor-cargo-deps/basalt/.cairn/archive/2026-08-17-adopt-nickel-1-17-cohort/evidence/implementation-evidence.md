# Nickel 1.17 cohort evidence

## Baseline

Basalt `main` at `19b3fd3946e65e63606a0922cd73b4d01f1bbd19` passed the direct release verifier and every Nix check before this change.

The baseline included 195 no-default tests, 208 CLI-feature tests, two doctests, formatting, Clippy, package checks, source checks, and documentation checks.

## Exact cohort

- Version: `1.17.0`
- Upstream revision: `1320a983e6c3d1e2fb53dd2464b084b4903b1426`
- Evaluator identity: `nickel-lang-cli nickel 1.17.0 (rev 1320a98)`
- Flake input: `nickel-1-17-0`

The lock file was generated with `nix flake lock --update-input nickel-1-17-0`. It was not edited by hand.

Every development, policy, fixture, freshness, and release use now selects `nickelCohort`. The exact-cohort check rejects `pkgs.nickel`, wrong revisions, wrong executable paths, and wrong version output.

## Compatibility

Four positive fixtures cover contracts, exports, imports, and merges.

Six negative fixtures reject contract violations, duplicate fields, malformed source, missing imports, type mismatches, and unknown fields.

Checked-in policy JSON and fixture JSON were regenerated with the exact evaluator. The existing policy freshness rail remains authoritative.

## Authority boundary

The runtime test confirms that an exported and parsed policy still denies a request without UCAN capability authority.

An exact capability remains required before Basalt allows the request. Nickel does not run during runtime enforcement.

## Release evidence

The direct release verifier checks `nickel --version` before policy freshness work.

It writes `nickel-toolchain.json` with the exact version, revision, evaluator identity, authority owner, and bounded non-claim.

## Validation

- The exact evaluator reported `nickel-lang-cli nickel 1.17.0 (rev 1320a98)`.
- Four positive and six negative Nickel fixtures passed their expected dispositions.
- Policy freshness passed with regenerated output.
- The direct release verifier passed 196 no-default tests and 209 CLI-feature tests.
- Formatting, two doctests, property tests, package checks, and both Clippy scopes passed.
- Every repository Nix check passed with local builders.
- Cairn repository, proposal, and design validation passed.

## Non-claims

Evaluator alignment does not prove policy correctness, UCAN correctness, key trust, external state truth, effect correctness, or release eligibility.
