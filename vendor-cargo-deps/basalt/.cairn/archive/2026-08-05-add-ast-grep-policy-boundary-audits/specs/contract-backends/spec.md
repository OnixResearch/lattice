# Contract Backends Specification Delta

## ADDED Requirements

### Requirement: Policy-boundary audit profiles

r[basalt.ast_grep_policy_audits.profile]

Basalt MUST define ast-grep audit profiles for syntax-level policy-boundary checks with declared scan scope, release posture, severity, and non-claims.

#### Scenario: Policy audit is advisory

- GIVEN a Basalt ast-grep policy-boundary rule is configured as advisory
- WHEN the release verifier runs the scan
- THEN findings MUST be reported as candidate boundary evidence and MUST NOT block release by default.

### Requirement: Receipt identity

r[basalt.ast_grep_policy_audits.identity]

Basalt MUST bind ast-grep tool identity, rule bundle BLAKE3 identity, scan scope, release verifier run identity, findings summary, and non-claims into audit receipts.

#### Scenario: Receipt references stale rule bundle

- GIVEN a release receipt references an ast-grep rule bundle identity
- WHEN the rule bundle changes before review
- THEN Basalt MUST require a fresh scan receipt before treating the audit as current.

### Requirement: Positive and negative fixtures

r[basalt.ast_grep_policy_audits.fixtures]

Basalt MUST require positive and negative fixtures before an ast-grep policy-boundary rule can become release-blocking.

#### Scenario: Blocking rule lacks negative fixture

- GIVEN a policy-boundary rule is proposed for blocking posture without a negative fixture
- WHEN Basalt validates the rule pack
- THEN validation MUST fail closed and keep the rule advisory.

### Requirement: Release receipt reporting

r[basalt.ast_grep_policy_audits.release_receipts]

Basalt SHOULD report ast-grep audit outcomes in release verification receipts as structural candidate evidence with remediation guidance and explicit non-claims.

#### Scenario: Enforcement bypass candidate is reported

- GIVEN ast-grep reports a candidate direct bypass around the enforcement helper
- WHEN the release receipt is emitted
- THEN the receipt SHOULD identify the rule, span, severity, and remediation note while preserving that authorization was not proven or disproven.

### Requirement: Shell-owned scan execution

r[basalt.ast_grep_policy_audits.shell_boundary]

Basalt MUST keep ast-grep process execution, filesystem scans, and raw output capture in release verifier or CLI shell code.

#### Scenario: Core enforcement runs

- GIVEN core enforcement evaluates a UCAN-backed request
- WHEN policy validation executes
- THEN it MUST NOT depend on ast-grep state, filesystem scans, process execution, or ambient scan results.

### Requirement: Audit validation evidence

r[basalt.ast_grep_policy_audits.validation]

Basalt MUST validate ast-grep policy audits with rule tests, fixture scans, release verifier receipt checks, Cairn gates, and focused release rails.

#### Scenario: Audit change is reviewed

- GIVEN ast-grep policy-boundary audit behavior changes
- WHEN validation evidence is assembled
- THEN evidence MUST include positive fixtures, negative fixtures, release receipt output, lifecycle gates, and focused Basalt verifier results.
