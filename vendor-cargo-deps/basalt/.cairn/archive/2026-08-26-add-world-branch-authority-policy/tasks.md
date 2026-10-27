## Phase 1: Policy and pure core

- [x] [serial] Record baseline Nickel policy freshness, UCAN composition, runtime-effect permission, receipt, and release-verification checks. r[contract_backends.world_branch_authority.verification]
- [x] [serial] Define typed Nickel contracts for branch actions, capability classes, scopes, modes, limits, currentness, rules, profiles, decisions, obligations, diagnostics, and non-claims. r[contract_backends.world_branch_authority.policy]
- [x] [depends:world-branch-authority-policy] Export checked generated JSON and add exact Rust DTO parity validation. r[contract_backends.world_branch_authority.policy]
- [x] [depends:world-branch-authority-policy] Implement pure deny-by-default validation, deterministic matching, ambiguity rejection, widening checks, optional UCAN composition, and obligation construction. r[contract_backends.world_branch_authority.decision] r[contract_backends.world_branch_authority.currentness]

## Phase 2: Evidence and consumer mapping

- [x] [depends:world-branch-authority-decision] Add metadata-only domain-separated BLAKE3 decision receipts with policy, request, selected rule, authority input, mode, obligation, diagnostic, and non-claim identities. r[contract_backends.world_branch_authority.evidence]
- [x] [depends:world-branch-authority-decision] Add frozen Molten mappings that retain native derivation, transfer, adapter, activation, promotion, and effect enforcement ownership. r[contract_backends.world_branch_authority.consumer]
- [x] [parallel] Add positive fixtures for copyable, narrower attenuation, linear obligation, simulation-only, promotion-gated, replacement, and non-branchable decisions. r[contract_backends.world_branch_authority.verification]
- [x] [parallel] Add negative unknown mode, ambiguous rules, wider destination, missing UCAN, stale currentness, linear copy, simulation live fallback, promotion bypass, lossy mapping, bearer material, and receipt-as-enforcement fixtures. r[contract_backends.world_branch_authority.verification]

## Phase 3: Documentation and gates

- [x] [serial] Document policy ownership, normalized inputs, mode semantics, consumer obligations, currentness rechecks, secret exclusions, and enforcement non-claims. r[contract_backends.world_branch_authority.boundary] r[contract_backends.world_branch_authority.evidence]
- [x] [depends:world-branch-authority-verification] Run policy export freshness, focused Rust and Nickel tests, frozen consumer compatibility, Clippy with warnings denied, Cairn validation and gates, release verification, and relevant Nix checks. r[contract_backends.world_branch_authority.verification]
