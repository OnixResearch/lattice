# Crate Consumability Specification Delta

## ADDED Requirements

### Requirement: Basalt CLI policy filesystem authority is capability-rooted

r[crate_consumability.cap_std_cli_policy_boundary.root_wrappers]

Basalt MUST represent CLI policy input, generated policy, and receipt/output filesystem authority as typed capability roots opened by CLI-feature code.

#### Scenario: Valid CLI policy path stays under the declared root

r[crate_consumability.cap_std_cli_policy_boundary.tests.positive]

- GIVEN Basalt has opened an operator-declared policy or output root
- WHEN CLI code reads a valid relative policy path, uses packaged policy fallback, or writes a receipt/output through the typed root
- THEN the operation MUST remain confined to that declared root while preserving existing valid-input behavior.

#### Scenario: Invalid CLI policy path cannot escape the root

r[crate_consumability.cap_std_cli_policy_boundary.tests.negative]

- GIVEN input names `../` traversal, an absolute path, a missing capability root, a symlink escape attempt, or an output escape attempt
- WHEN Basalt resolves the path through the typed root
- THEN Basalt MUST fail closed before reading policy bytes or writing receipt/output files outside the declared root.

### Requirement: Capability dependency stays outside no-default library APIs

r[crate_consumability.cap_std_cli_policy_boundary.dependency]

Basalt MUST add `cap-std` only to CLI-feature code that owns filesystem effects and MUST preserve no-default library builds without CLI-only filesystem dependencies.

#### Scenario: Library core remains filesystem-neutral

r[crate_consumability.cap_std_cli_policy_boundary.conversion]

- GIVEN enforcement core, UCAN validation, Nickel/Steel backend DTOs, and packaged policy helpers are reviewed
- WHEN cap-std adoption is complete
- THEN those library surfaces MUST operate on in-memory policy data, DTOs, or validated relative locators and MUST NOT depend on `cap-std` or open ambient filesystem paths directly.

### Requirement: Capability boundary is documented without policy overclaims

r[crate_consumability.cap_std_cli_policy_boundary.docs]

Basalt MUST document that capability roots bound local CLI filesystem authority only and do not prove policy truth, UCAN issuer trust, caveat semantics, or downstream deployment readiness.

#### Scenario: Closeout evidence includes CLI policy boundary validation

r[crate_consumability.cap_std_cli_policy_boundary.validation]

- GIVEN the cap-std CLI-policy change is ready to archive
- WHEN focused CLI, no-default library, Cairn validation, and gate checks run
- THEN the evidence MUST include positive and negative filesystem-boundary coverage and visible policy/UCAN non-claim documentation.
