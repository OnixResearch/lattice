## Phase 1: Baseline evidence and gate repair

- [x] [serial] V1 Record the failing deny-level Cargo/Octet status, old and current config identities, exact 1,141-record comparison, and stale traceability count/hash. r[lattice.repository_validation.baselined_octet_gate]
- [x] [depends:V1] I1 Change every configured Octet lint from deny to warning observation without allowing or disabling any lint, then generate a complete runner-owned candidate baseline. r[lattice.repository_validation.baselined_octet_gate.scenario.accepted]
- [x] [depends:I1] I2 Add a permanent isolated negative Nix fixture that injects a new stable finding and requires canonical wrapper rejection. r[lattice.repository_validation.baselined_octet_gate.scenario.new_finding]

## Phase 2: Traceability inventory and evidence

- [x] [parallel] I3 Bind the typed traceability policy to the exact 12-item missing-set identity and regenerate the deterministic inventory without promoting coverage. r[lattice.repository_validation.traceability_debt_inventory]
- [x] [depends:I3] I4 Refresh the accepted repository-validation completion byte identities for the Radicle cutover without changing review decisions or coverage. r[lattice.repository_validation.repository_checks_completion.scenario.exact]
- [x] [depends:I1] [depends:I2] [depends:I3] D1 Update operator documentation and record BLAKE3-bound positive, negative, and non-claim evidence. r[lattice.repository_validation.baselined_octet_gate.scenario.boundary]

## Phase 3: Validation and lifecycle

- [x] [depends:D1] V2 Run focused positive/negative checks, repository checks, Cairn proposal/design/tasks gates, and bounded full flake validation; sync the accepted requirement and archive only after deterministic evidence agrees. r[lattice.repository_validation.baselined_octet_gate]
