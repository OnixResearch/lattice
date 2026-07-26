## ADDED Requirements

### Requirement: External graph source requires Radicle evidence
r[lattice.execution_graph_external_adoption.source] Lattice MUST admit `execution-graph-core` only from one licensed immutable revision of the canonical Radicle project, with aligned Cargo and Nix source identity and no GitHub, sibling-path, or floating fallback.

#### Scenario: Producer source is complete
r[lattice.execution_graph_external_adoption.source.scenario.accepted]
- GIVEN a verified RID, authority policy, exact Git object, source archive identity, admitted endpoint, license, and no-fallback evidence
- WHEN dependency admission runs
- THEN Lattice MAY begin a reversible compatibility pilot

#### Scenario: Producer source is incomplete
r[lattice.execution_graph_external_adoption.source.scenario.rejected]
- GIVEN any required identity, license, endpoint, revision, or negative acquisition fact is absent or contradictory
- WHEN dependency admission runs
- THEN Lattice MUST reject the source before changing production dependency authority

### Requirement: External graph decisions preserve compatibility
r[lattice.execution_graph_external_adoption.compatibility] Lattice MUST dual-run positive and negative graph admission, ordering, readiness, lifecycle, resume, critical-path, and bottleneck fixtures before removing its vendored source.

#### Scenario: Vendor and external results agree
r[lattice.execution_graph_external_adoption.compatibility.scenario.agree]
- GIVEN identical reviewed graph and lifecycle facts
- WHEN both implementations run
- THEN admitted results and deterministic diagnostics MUST agree

#### Scenario: Results or sources differ
r[lattice.execution_graph_external_adoption.compatibility.scenario.differ]
- GIVEN a result, diagnostic, source object, package identity, or Cargo/Nix revision differs
- WHEN cutover validation runs
- THEN cutover MUST stop until the difference is reviewed and resolved

### Requirement: Lattice retains product authority
r[lattice.execution_graph_external_adoption.authority] Lattice MUST retain node, port, value, effect, scheduler, retry, trigger, orchestration, persistence, artifact, diagnostic, evidence, and release authority after adopting the external graph core.

#### Scenario: Shared core requests product authority
r[lattice.execution_graph_external_adoption.authority.scenario.denied]
- GIVEN an adapter or dependency change moves scheduling, execution, retries, resources, persistence, receipts, or product node types into the graph core
- WHEN architecture validation runs
- THEN the change MUST be rejected before cutover

### Requirement: Cutover removes duplicate source without fallback
r[lattice.execution_graph_external_adoption.cutover] Lattice MUST remove vendored graph source only after exact parity and MUST use VCS rollback rather than a hidden runtime or acquisition fallback.

#### Scenario: Cutover passes
r[lattice.execution_graph_external_adoption.cutover.scenario.accepted]
- GIVEN source alignment, compatibility, architecture, focused product, and repository checks pass
- WHEN cutover is accepted
- THEN Lattice MAY remove the vendor and sync paths while recording the exact rollback commit

#### Scenario: Passing graph checks are overclaimed
r[lattice.execution_graph_external_adoption.cutover.scenario.non-claim]
- GIVEN standalone and consumer graph checks pass
- WHEN evidence is reported
- THEN it MUST NOT claim workflow correctness, scheduler fairness, effect safety, persistence correctness, Radicle availability, or release readiness
