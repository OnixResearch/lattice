## ADDED Requirements

### Requirement: Exact implementation child lineage

r[lattice.animus_implementation_profile] Lattice must support a distinct opt-in Animus implementation profile with bounded delegation, unchanged OMP defaults and exact native lineage readback.

#### Scenario: Exact child receipt

- GIVEN an explicitly selected attempt and a genuine child lifecycle observation
- WHEN the owner exports lineage for the exact bound request
- THEN the authorized consumer can display the parent and child without private content

#### Scenario: Crossed or missing receipt

- GIVEN a missing or mismatched observation
- WHEN the consumer reads lineage
- THEN it must not report a child fact from that observation
