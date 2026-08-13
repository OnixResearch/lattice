## Design

1. Define ast-grep audit profiles for Basalt policy-boundary source shapes: direct bypass of enforcement helpers, ambient authority in policy cores, Steel adapter host capability grants, CLI-only effects leaking into core, and stale generated-policy consumption.
2. Keep Nickel policy source and generated JSON freshness checks authoritative. ast-grep can only flag source shapes around those boundaries.
3. Bind scan receipts to ast-grep version, rule bundle BLAKE3 identity, repository scope, release verifier run identity, and non-claims.
4. Require positive and negative fixtures before a rule can move from advisory to release-blocking.
5. Report findings in release verification summaries as structural boundary audit results with explicit remediation guidance.
6. Keep all process execution and filesystem scanning in release verifier shell code; pure policy/enforcement cores consume no ast-grep state.
7. Preserve downstream compatibility by making ast-grep release gates opt-in or ratcheted through existing release verifier policy.

## Non-Goals

- No replacement for Nickel policy contracts, generated policy freshness, UCAN proof-chain verification, or Steel adapter DTO validation.
- No claim that clean ast-grep scans authorize a request or prove policy safety.
- No automatic rewrites of policy/enforcement code during release verification.
