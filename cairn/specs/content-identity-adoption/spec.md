# Content Identity Adoption Specification

## Purpose

Defines the `content-identity-adoption` capability.

## Requirements

### Requirement: Shared identity source is exact and Radicle-backed
r[lattice.content_identity_adoption.source] Lattice MUST admit `content-identity-core` only from one licensed immutable revision of its canonical Radicle project, with aligned Cargo and Nix identity and no GitHub, sibling, or floating fallback.

#### Scenario: Producer source is accepted
r[lattice.content_identity_adoption.source.scenario.accepted]
- GIVEN verified source, license, RID, authority, exact revision, framing boundary, endpoint, and no-fallback evidence
- WHEN dependency admission runs
- THEN Lattice MAY begin a reversible identity-family pilot

#### Scenario: Producer source is incomplete
r[lattice.content_identity_adoption.source.scenario.rejected]
- GIVEN any required source, license, boundary, endpoint, revision, or negative acquisition fact is absent or contradictory
- WHEN dependency admission runs
- THEN Lattice MUST reject adoption before production identity behavior changes

### Requirement: Lattice retains identity roles and canonical inputs
r[lattice.content_identity_adoption.boundary] Lattice MUST retain every product domain registry, canonical projection, compatibility decision, persistence rule, and claim interpretation while delegating only eligible unkeyed framing mechanics.

#### Scenario: Security or evidence role enters shared framing
r[lattice.content_identity_adoption.boundary.scenario.denied]
- GIVEN a migration moves vault streams, signatures, credentials, password hashing, KDFs, authentication, Valence evidence roles, product domains, or canonical projections into the shared core
- WHEN architecture validation runs
- THEN the migration MUST be rejected or narrowed

### Requirement: Every eligible identity has fixed compatibility vectors
r[lattice.content_identity_adoption.compatibility] Lattice MUST inventory and dual-run fixed positive and negative vectors before replacing any current identity framing and MUST version any deliberate identity change explicitly.

#### Scenario: Legacy and shared digest agree
r[lattice.content_identity_adoption.compatibility.scenario.agree]
- GIVEN unchanged role, domain, canonicalization, fields, order, values, and output spelling
- WHEN legacy and shared paths run
- THEN the complete admitted identity MUST remain identical

#### Scenario: Digest changes without migration
r[lattice.content_identity_adoption.compatibility.scenario.differ]
- GIVEN a digest, metadata field, persisted key, or acceptance result differs without a new reviewed version
- WHEN compatibility validation runs
- THEN cutover MUST stop before stored or wire identities change

### Requirement: Identity cutover is incremental and reversible
r[lattice.content_identity_adoption.cutover] Lattice MUST replace only admitted identity families, remove duplicate helpers only after complete caller coverage, and retain VCS rollback without runtime or acquisition fallback.

#### Scenario: One identity family passes
r[lattice.content_identity_adoption.cutover.scenario.accepted]
- GIVEN exact source alignment, complete inventory, vector parity, role guards, focused product tests, and repository checks pass
- WHEN that family is accepted
- THEN Lattice MAY remove its duplicate framing helper while retaining domain and canonicalization authority

#### Scenario: Passing digest vectors are overclaimed
r[lattice.content_identity_adoption.cutover.scenario.non-claim]
- GIVEN selected digest vectors pass
- WHEN evidence is reported
- THEN it MUST NOT claim semantic equality, canonicalization correctness, confidentiality, authenticity, authorization, cache safety, deduplication safety, or release readiness
