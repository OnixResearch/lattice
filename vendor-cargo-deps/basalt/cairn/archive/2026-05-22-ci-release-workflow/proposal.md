# Add CI release workflow

## Why

The crate now has a maintained local release verifier and all prior crate-consumability Cairns are archived, but there is no checked-in hosted CI workflow and this checkout has no configured remote. Downstream consumers and maintainers need a visible, repeatable release-readiness path outside the current machine.

The current Nix flake release check intentionally records Cargo-dependent rails as deferred in sandbox mode because the crate depends on a private UCAN Git source. A CI workflow must make that boundary explicit: it can always run the sandbox-safe flake checks, and it must document or wire the full direct verifier only for environments with private dependency access.

## What changes

- Add a checked-in CI workflow definition for release-readiness checks.
- Make the workflow run `nix flake check -L` as the default public/sandbox-safe gate.
- Document the private Git dependency requirement for running the full direct release verifier in CI.
- Avoid committing credentials, tokens, or private key material.
- Add docs that distinguish sandbox-safe CI evidence from full release verification evidence.

## Scope

In scope: CI workflow files, README/release docs, verifier invocation documentation, and crate-consumability requirements for CI evidence. Out of scope: adding a git remote, pushing to GitHub, provisioning secrets, publishing crates, changing UCAN dependency semantics, or weakening the release verifier.

## Validation

Validate this package with Cairn proposal/design/tasks gates. Implementation must run `cairn validate --root .`, `git diff --check`, `nix flake check -L`, and a syntax/static review of any workflow file before archive.
