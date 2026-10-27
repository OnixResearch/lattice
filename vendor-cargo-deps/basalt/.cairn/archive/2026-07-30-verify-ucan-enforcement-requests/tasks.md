## Source Boundary

UCAN commit `34449ba1f1a9ef991225e4e6683b5d6aca8c07cf` satisfies the dependency criteria.
Basalt uses a reviewed vendored snapshot because the revision is not remotely fetchable.

## Phase 1: Contract and API

- [x] [serial] Add the deterministic vendored UCAN import, revision, BLAKE3 manifest, license check, and Nix check. r[basalt.ucan_token_flow.verify_ucan_enforcement.contract]
- [x] [serial] Define the raw-invocation adapter, injected boundaries, live admission capability, typed outcomes, and non-claims. r[basalt.ucan_token_flow.verify_ucan_enforcement.contract]
- [x] [serial] [depends:basalt.ucan_token_flow.verify_ucan_enforcement.contract] Implement the strongest pinned UCAN verification composition without ambient effects. r[basalt.ucan_token_flow.verify_ucan_enforcement.verification]
- [x] [serial] [depends:basalt.ucan_token_flow.verify_ucan_enforcement.verification] Build policy authority before replay and feed only adapter-produced facts through `adapt_enforcement_request` and `evaluate_authority`. r[basalt.ucan_token_flow.verify_ucan_enforcement.authority]
- [x] [serial] [depends:basalt.ucan_token_flow.verify_ucan_enforcement.authority] Implement deterministic redacted BLAKE3 receipts that cannot recreate authority. r[basalt.ucan_token_flow.verify_ucan_enforcement.receipt]

## Phase 2: Negative and Positive Evidence

- [x] [serial] [depends:basalt.ucan_token_flow.verify_ucan_enforcement.verification] Add positive direct and delegated invocation tests. r[basalt.ucan_token_flow.verify_ucan_enforcement.tests.positive]
- [x] [serial] [depends:basalt.ucan_token_flow.verify_ucan_enforcement.verification] Add malformed, oversized, signature, proof, issuer, holder, time, caveat, scope, revocation, replay, limit, and policy tests. r[basalt.ucan_token_flow.verify_ucan_enforcement.tests.negative]
- [x] [serial] [depends:basalt.ucan_token_flow.verify_ucan_enforcement.receipt] Add receipt redaction, deterministic identity, and persisted-receipt non-authority tests. r[basalt.ucan_token_flow.verify_ucan_enforcement.tests.receipt]
- [x] [serial] [depends:basalt.ucan_token_flow.verify_ucan_enforcement.tests.negative] Prove that unverifiable input produces no authority decision. r[basalt.ucan_token_flow.verify_ucan_enforcement.tests.no_decision]

## Phase 3: Documentation and Validation

- [x] [serial] [depends:basalt.ucan_token_flow.verify_ucan_enforcement.tests.receipt] Document API use, ownership boundaries, and bounded claims. r[basalt.ucan_token_flow.verify_ucan_enforcement.docs]
- [x] [serial] [depends:basalt.ucan_token_flow.verify_ucan_enforcement.docs] Run focused tests, format, Clippy, workspace tests, release checks, and canonical-policy Cairn gates. r[basalt.ucan_token_flow.verify_ucan_enforcement.validation]
- [x] [serial] [depends:basalt.ucan_token_flow.verify_ucan_enforcement.validation] Sync the accepted specification and archive the completed change. r[basalt.ucan_token_flow.verify_ucan_enforcement.archive]
