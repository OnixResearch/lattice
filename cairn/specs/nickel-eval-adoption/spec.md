# Nickel Eval Adoption Specification

## Purpose

Defines the `nickel-eval-adoption` capability.

## Requirements

### Requirement: Shared evaluator source is exact and Radicle-backed
r[lattice.nickel_eval_adoption.source] Lattice MUST admit `nickel-eval` only from one licensed immutable revision of its canonical Radicle project, with aligned Cargo and Nix identity, a supported Nickel cohort, and no GitHub, sibling, or floating fallback.

#### Scenario: Producer source and cohort are accepted
r[lattice.nickel_eval_adoption.source.scenario.accepted]
- GIVEN verified source, license, RID, authority, exact revision, features, supported Nickel cohort, endpoint, and no-fallback evidence
- WHEN dependency admission runs
- THEN Lattice MAY begin a reversible evaluator pilot

#### Scenario: Producer source or cohort is incomplete
r[lattice.nickel_eval_adoption.source.scenario.rejected]
- GIVEN any required source, license, version, feature, endpoint, or negative acquisition fact is absent or contradictory
- WHEN dependency admission runs
- THEN Lattice MUST reject adoption before production dependency authority changes

### Requirement: Lattice retains product evaluation policy
r[lattice.nickel_eval_adoption.boundary] Lattice MUST retain configuration discovery, product contracts, target decoding, defaults, warnings, safe diagnostics, store-opening order, mutation behavior, and release policy while delegating only admitted evaluator mechanics.

#### Scenario: Shared package gains product policy
r[lattice.nickel_eval_adoption.boundary.scenario.denied]
- GIVEN an adapter moves Lattice contracts, defaults, workflow types, fallback, secrets, CLI exits, persistence, or release decisions into the shared package
- WHEN architecture validation runs
- THEN adoption MUST be rejected or narrowed

### Requirement: Every evaluator call site proves compatibility
r[lattice.nickel_eval_adoption.compatibility] Lattice MUST dual-run positive and negative fixtures for every inventoried evaluator call site and MUST classify all output, diagnostic, redaction, or failure differences.

#### Scenario: Legacy and shared paths agree
r[lattice.nickel_eval_adoption.compatibility.scenario.agree]
- GIVEN identical source, imports, contract material, and target decoding
- WHEN legacy and shared paths run
- THEN accepted values and stable safe diagnostic classes MUST agree

#### Scenario: Paths differ
r[lattice.nickel_eval_adoption.compatibility.scenario.differ]
- GIVEN an accepted value, error class, source location, redaction result, or fallback behavior differs
- WHEN compatibility validation runs
- THEN cutover MUST stop until the difference is explicitly resolved

### Requirement: Evaluator cutover is single-source and reversible
r[lattice.nickel_eval_adoption.cutover] Lattice MUST use one production evaluator path after acceptance, remove duplicate mechanics only after parity, and retain VCS rollback without runtime or acquisition fallback.

#### Scenario: Cutover passes
r[lattice.nickel_eval_adoption.cutover.scenario.accepted]
- GIVEN source alignment, compatibility, architecture, focused product, and repository checks pass
- WHEN cutover is accepted
- THEN Lattice MAY remove duplicate evaluator mechanics and record the exact rollback state

#### Scenario: Passing evaluation is overclaimed
r[lattice.nickel_eval_adoption.cutover.scenario.non-claim]
- GIVEN evaluator and consumer checks pass
- WHEN evidence is reported
- THEN it MUST NOT claim Nickel soundness, evaluator equivalence, import-closure completeness, contract correctness, configuration safety, or release readiness
