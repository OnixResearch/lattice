# Repository Validation

## ADDED Requirements

### Requirement: Pure internal core crate
r[lattice.architecture.core_crate] Lattice SHALL keep selected deterministic workflow decisions in an internal `lattice-core` crate with no host-effect authority.

#### Scenario: Pure decisions remain isolated
r[lattice.architecture.core_crate.pure]
- GIVEN value, expression, control-flow, and content-identity inputs
- WHEN the core crate computes a result
- THEN it performs no filesystem, process, network, persistence, clock, provider, or terminal effect

#### Scenario: Existing public paths remain compatible
r[lattice.architecture.core_crate.compatibility]
- GIVEN a caller uses the existing `lattice` module paths
- WHEN the caller builds after extraction
- THEN explicit re-exports provide the same public types and functions
- AND focused behavior tests produce the same results

#### Scenario: Forbidden dependency is rejected
r[lattice.architecture.core_crate.dependencies]
- GIVEN a candidate `lattice-core` manifest imports a host, persistence, network, process, TUI, or concrete runtime dependency
- WHEN the repository dependency check runs
- THEN it rejects the manifest with the forbidden dependency name

#### Scenario: Core source remains traceable
r[lattice.architecture.core_crate.traceability]
- GIVEN direct requirement markers move into `crates/lattice-core`
- WHEN traceability coverage runs
- THEN it discovers the core markers
- AND it reports no missing or dangling requirement

#### Scenario: Validation remains complete
r[lattice.architecture.core_crate.validation]
- GIVEN the extraction and compatibility facade
- WHEN repository validation runs
- THEN focused parity, core, dependency, Octet, Cairn, Clippy, and Nix checks pass
