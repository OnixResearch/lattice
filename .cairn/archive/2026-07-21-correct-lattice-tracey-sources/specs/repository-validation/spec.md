# Repository validation delta

## ADDED Requirements

### Requirement: Repository-owned traceability sources
r[lattice.repository_validation.traceability_sources] Lattice MUST evaluate accepted requirement linkage against bounded repository-owned source roots and MUST preserve fail-closed missing and dangling diagnostics.

#### Scenario: Lattice markers are discovered
r[lattice.repository_validation.traceability_sources.scenario.discovery]
- GIVEN accepted requirements live under `cairn/specs` and Lattice-owned markers live under the configured source, test, example, script, and policy roots
- WHEN an operator runs `cairn tracey coverage --root .` with the checked repository policy
- THEN the command MUST discover those markers and MUST NOT fall back to unrelated `crates` or `tools` roots

#### Scenario: Missing and dangling linkage remains fail closed
r[lattice.repository_validation.traceability_sources.scenario.fail-closed]
- GIVEN correct source discovery reveals accepted requirements without markers or markers without accepted requirements
- WHEN Tracey computes coverage
- THEN it MUST report the missing and dangling identifiers, render `valid: false`, and exit unsuccessfully

#### Scenario: Unsafe source roots are rejected
r[lattice.repository_validation.traceability_sources.scenario.unsafe-root]
- GIVEN a policy fixture configures an absolute, parent-traversing, or otherwise unsafe traceability root
- WHEN Cairn validates the Nickel policy export
- THEN export MUST fail before the root can contribute requirement evidence
