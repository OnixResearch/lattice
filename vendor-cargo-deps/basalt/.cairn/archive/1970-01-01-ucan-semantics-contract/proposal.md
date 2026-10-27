## Why

Basalt enforces a Nickel/Steel policy decision against UCAN capabilities, but its dependency contract currently names a UCAN Git revision rather than the exact UCAN authorization semantics Basalt relies on. That leaves wildcard handling, caveat behavior, proof traversal, replay hooks, WOT helpers, and error boundaries coupled to UCAN implementation details.

This change adds a Basalt-facing UCAN semantics contract so Basalt can validate the UCAN authorization model it consumes before claiming policy enforcement compatibility.

## What Changes

- Add a UCAN semantics contract fixture set for Basalt's enforcement boundary.
- Bind Basalt policy decisions to explicit UCAN behavior: resource/ability containment, wildcard segment rules, caveat pass/fail semantics, proof traversal limits, replay denial handling, and structured error classes.
- Add positive fixtures for matching policy+UCAN allow behavior and negative fixtures for over-delegation, wildcard request misuse, missing caveat policy, stale proof semantics, and replay denial.
- Preserve non-claims: Basalt checks policy/UCAN compatibility only; it does not prove consumer backend freshness, key-resolution truth, cryptographic soundness beyond UCAN's own boundary, or application-level authorization completeness.

## Impact

- **Basalt** gets a reviewed contract for the UCAN authorization semantics it supports.
- **UCAN** can evolve internals while preserving a fixture-backed consumer contract.
- **Downstream applications** see clearer compatibility diagnostics when UCAN behavior changes.
- **Testing** adds positive and negative contract fixtures plus focused enforcement tests.
