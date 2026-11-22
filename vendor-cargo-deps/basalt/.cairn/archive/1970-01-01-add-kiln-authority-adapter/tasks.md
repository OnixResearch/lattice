## Phase 1: Contract and binding

- [x] [serial] Add the nine closed Kiln operations with reviewed contract ids, abilities and the `kiln://implementation/v1/` prefix, and extend `EnforcementAction`. r[basalt.kiln_authority.actions]
- [x] [serial] [depends:basalt.kiln_authority.actions] Implement the framed canonical binding, field admission, candidate rule and bounded resource. r[basalt.kiln_authority.binding]
- [x] [serial] [depends:basalt.kiln_authority.binding] Add the nine Kiln contracts to `contracts/policy.ncl` and regenerate the packaged policy artifact. r[basalt.kiln_authority.policy]

## Phase 2: Verified adapter

- [x] [serial] [depends:basalt.kiln_authority.binding] Implement `Verifier::verify` over the pinned UCAN ingress, returning only the opaque session. r[basalt.kiln_authority.ingress]
- [x] [serial] [depends:basalt.kiln_authority.ingress] Implement non-consuming `Verifier::revalidate` and session operation checks. r[basalt.kiln_authority.revalidation]
- [x] [serial] [depends:basalt.kiln_authority.ingress] Document ownership, redaction and non-claims for the adapter. r[basalt.kiln_authority.ingress]

## Phase 3: Evidence

- [x] [serial] [depends:basalt.kiln_authority.revalidation] Add positive fixtures for every operation, exact delegation and revalidation. r[basalt.kiln_authority.delegation.scenario.exact] r[basalt.kiln_authority.revalidation.scenario.nonconsuming]
- [x] [serial] [depends:basalt.kiln_authority.revalidation] Add negative fixtures for field substitution, candidate misuse, expiry, revocation, stale generation, untrusted roots and widening. r[basalt.kiln_authority.binding.scenario.substitution] r[basalt.kiln_authority.binding.scenario.candidate] r[basalt.kiln_authority.revalidation.scenario.stale] r[basalt.kiln_authority.delegation.scenario.widen]
- [x] [serial] [depends:basalt.kiln_authority.actions] Assert the packaged policy carries every Kiln contract. r[basalt.kiln_authority.policy.scenario.packaged]
- [x] [serial] [depends:basalt.kiln_authority.revalidation] Run focused Cargo tests, format, Clippy, the workspace suite and Cairn validation. r[basalt.kiln_authority.ingress]
