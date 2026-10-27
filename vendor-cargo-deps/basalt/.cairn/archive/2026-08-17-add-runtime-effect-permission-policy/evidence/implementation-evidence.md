# Runtime effect permission evidence

## Baseline

Basalt `main` at `28b148f0785d59f98e008a1670bf772c25e7a081` passed the direct release verifier and every Nix check before implementation.

The baseline included 201 no-default tests, 214 CLI-feature tests, policy freshness, formatting, Clippy, package checks, and documentation checks.

## Bounded agentOS reference

- Repository: `https://github.com/rivet-dev/agentos`
- Revision: `44a6c8022f26ef0edd56e3d224f8296ee6387232`
- Permission type blob: `e38172cfa97b21922fafcb822d254929bf1c191f`
- Evaluation blob: `1ab663a8a2f8bab31218f39c49193bd63d31f82e`
- Default-deny test blob: `0fd804e703fe38d50efd353514ebea16f36452ac`
- License blob: `f23ef9d48af638ddf215b378916767349a9591a8`
- License: `Apache-2.0`

The reference supplies separate scopes, operation rules, and default-deny comparison facts. Basalt copies no schema, glob behavior, runtime, or sandbox claim.

## Implementation

The pure core supports closed filesystem, network, process, environment, and binding scopes.

Each rule declares an operation, exact or prefix match, normalized subject pattern, priority, and decision. Equal-priority conflicting rules deny as ambiguous.

The default profile denies by default. It requires a current lossless adapter, an admitted Basalt contract decision, and an opaque verified UCAN input.

Human approval is a separate observation. It cannot override denial or missing authority.

## Consumer fixtures

The dependency-free fixture matrix pins:

- Animus revision `4f08f89d652a6e6c83b2a41fd9d3c7651daf5d68`
- Molten revision `4734913c1c230a35fc121b948cdf54ac0619dd2a`

The fixtures record selected source paths and mapping examples only. They do not prove complete consumer mappings or native enforcement.

## Verification

Positive cases cover all five scopes and both match modes.

Negative cases cover unknown scope, unknown operation, unsafe defaults, duplicate and ambiguous rules, missing priority, raw host paths, stale profiles, stale references, weakened non-claims, lossy mapping, missing authority, denied approval, and unlisted subjects.

## Receipts

Receipts bind policy, request, selected rule, authority, approval, diagnostic, and reference identities with BLAKE3.

They omit credentials, environment values, bearer tokens, raw host paths, and effect payloads.

## Validation

- Exact Nickel policy and consumer mapping exports passed freshness checks.
- Eleven invalid Nickel fixtures failed closed.
- Six focused Rust tests covered positive and negative policy behavior.
- Strict Clippy passed for every target and feature with denied warnings.
- The direct release verifier passed 207 no-default tests and 220 CLI-feature tests.
- Formatting, two doctests, package checks, consumer checks, and documentation checks passed.
- Every repository Nix check passed with local builders.
- Cairn repository, proposal, and design validation passed.

## Non-claims

A policy allow or passing receipt does not execute an effect, provide a sandbox, prove current native enforcement, or grant release eligibility.
