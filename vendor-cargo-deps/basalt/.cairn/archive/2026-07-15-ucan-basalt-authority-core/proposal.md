## Why

Basalt binds Nickel-authored contract policy to UCAN-derived authorization facts. UCAN owns token parsing, proof chains, caveats, signatures, replay hooks, and verification shells; Basalt owns contract/resource/ability policy checks. The reusable seam is the pure authority decision DTO: verified grants plus Basalt request facts produce allow/deny diagnostics.

A shared `ucan-basalt-authority-core` lets Basalt and UCAN-facing consumers reuse that decision boundary without moving key resolution, clocks, revocation stores, replay stores, networking, or Nickel export into the core.

## What Changes

- Define pure authority input/output DTOs for contract ID, resource, ability, request ref, verified grant refs, UCAN verification receipt refs, Basalt policy refs, caveat/replay dispositions, and deny diagnostics.
- Add positive fixtures for matching Basalt policy plus verified UCAN grants.
- Add negative fixtures for missing grants, wrong resource, wrong ability, stale verification refs, caveat denial, replay denial, missing policy refs, and authority overclaims.
- Preserve Nickel as the human-authored contract policy boundary and Rust as runtime decision/receipt owner.
- Document the division between UCAN token verification and Basalt contract admission.

## Impact

- **Basalt** owns the shared authority decision DTO and policy admission core.
- **UCAN** remains owner of token/proof/caveat verification and resolver shell boundaries.
- **Consumers** get a stable pure interface for composed UCAN/Basalt authority checks.
