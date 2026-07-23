## ADDED Requirements

### Requirement: Reproducible bounded-exec source identity
r[lattice.repository_validation.bounded_exec_source] Lattice MUST consume `bounded-exec` only from one reviewed immutable published Git revision, MUST align Cargo and Nix source identity to that revision, and MUST reject sibling path substitution, source drift, or an unresolvable dependency.

#### Scenario: Published immutable source is admitted
r[lattice.repository_validation.bounded_exec_source.scenario.published]
- GIVEN the canonical `bounded-exec` repository exposes the reviewed revision and its AGPL source
- WHEN Cargo and Nix resolve Lattice's dependency closure
- THEN both MUST use that exact revision and repository validation MUST confirm the aligned identity

#### Scenario: Unpublished or substituted source fails closed
r[lattice.repository_validation.bounded_exec_source.scenario.invalid]
- GIVEN the canonical source is unpublished, the revision cannot resolve, Cargo and Nix identities differ, or a sibling path is substituted
- WHEN repository validation evaluates the dependency
- THEN it MUST fail before build or release evidence and MUST NOT treat the local checkout as a durable product input
