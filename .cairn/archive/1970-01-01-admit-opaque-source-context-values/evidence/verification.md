# Verification evidence

Date: 2026-08-20

## Focused evidence

- The contract rail passes 18 unit tests, 2 producer tests, and 2 source-boundary tests.
- The application workflow exchange rail passes nine tests.
- Mixed-case Radicle RID bytes remain exact.
- Empty, oversized, control-bearing, and delete-bearing source values fail closed.
- The typed Nickel contract accepts the mixed-case positive source fixture.
- Root and contract Clippy pass with `-D warnings`.
- Contract and transport Octet deny-all checks are clean.
- The root Octet gate reports zero new and zero stale identities.
- `scripts/check.rs` passes.
- `nix flake check path:$PWD -L --option secret-key-files ''` passes all checks, including 1,036 nextest cases.
- Cairn validation and proposal, design, and task gates pass.
- Tracey debt matches base exactly: 139 missing and 3 dangling, with no additions or removals.
- The Tracey receipt BLAKE3 is `7985c2d6d8e20947fac29a55d711b9e9891861d5ea43b403767a01ad61e6b744`.

## Claim boundary

This evidence proves bounded source-value construction, serialization, request identity framing, and local environment binding.

It does not prove source trust, revision availability, provider correctness, workflow correctness, effect success, deployment, or release readiness.
