## Why

Basalt hardens UCAN/Nickel/Steel policy boundaries. ast-grep can provide a fast syntax-aware audit layer for policy-enforcement call sites, shell/core separation, Steel adapter authority, and test-fixture drift. Basalt should use ast-grep as a bounded structural signal while keeping Nickel policy contracts, Rust enforcement, and UCAN verification authoritative.

## What Changes

- Define ast-grep policy-boundary audit profiles for Rust enforcement code, Nickel-generated artifact use, Steel adapter seams, and CLI shell authority.
- Add positive and negative fixtures before rules can become blocking release checks.
- Record ast-grep tool identity, rule bundle BLAKE3 hash, scan scope, findings, and non-claims in release verification receipts.
- Preserve ast-grep findings as candidate boundary evidence, not authorization proof.

## Impact

- **Surfaces**: contract backends, UCAN token flow, release verifier, CLI feature tests, downstream compatibility receipts.
- **Non-claims**: ast-grep does not prove UCAN authority, policy correctness, caveat satisfaction, Steel safety, or release readiness.
- **Validation**: ast-grep rule tests, policy-boundary fixtures, release verifier receipt checks, Cairn gates, and focused Basalt release rails.
