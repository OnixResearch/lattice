# Budget Ledger Incubation Specification

## Purpose

Defines the `budget-ledger-incubation` capability.

## Requirements

### Requirement: Budget ledger starts as a local functional core
r[lattice.budget_ledger_incubation.boundary] Lattice MUST isolate deterministic reserve, total, settle, breach, and remaining-balance decisions from orchestration I/O and policy shells before considering external publication.

#### Scenario: Ledger runs from supplied facts
r[lattice.budget_ledger_incubation.boundary.scenario.pure]
- GIVEN explicit named-dimension policy, limits, consumed totals, reservations, reservation identity, planned charge, and observed charge
- WHEN ledger decisions run
- THEN results MUST require no store, filesystem, environment, clock, process, network, provider, CLI, or event-hash authority

#### Scenario: Product policy enters the ledger
r[lattice.budget_ledger_incubation.boundary.scenario.overreach]
- GIVEN a candidate ledger reads provider usage, chooses unknown-outcome charges, authorizes actors, mutates persistence, emits events, or interprets release readiness
- WHEN architecture validation runs
- THEN the candidate MUST be rejected or narrowed

### Requirement: Local extraction preserves accounting compatibility
r[lattice.budget_ledger_incubation.compatibility] The local core MUST preserve current reservation, settlement, overflow, limit, breach, serialization, event, replay, retry, abandonment, and inspection behavior across positive and negative fixtures.

#### Scenario: Valid reservation and settlement agree
r[lattice.budget_ledger_incubation.compatibility.scenario.accepted]
- GIVEN a valid unique reservation within all enforced limits and an owned observed charge
- WHEN legacy and extracted paths reserve and settle
- THEN consumed totals, pending reservations, breach state, remaining balances, events, and safe inspection MUST agree

#### Scenario: Invalid accounting fails closed
r[lattice.budget_ledger_incubation.compatibility.scenario.rejected]
- GIVEN a required bound is missing, identity is duplicate or stale, settlement has no reservation, totals exceed limits, or arithmetic overflows
- WHEN legacy and extracted paths run
- THEN both MUST reject without partial accounting mutation

### Requirement: External promotion requires a real second consumer
r[lattice.budget_ledger_incubation.promotion] Lattice MUST NOT create or publish an independent budget-ledger repository until a second consumer proves equivalent durable reserve-and-settle semantics and two simplifying adapters preserve negative behavior.

#### Scenario: Only scalar limits overlap
r[lattice.budget_ledger_incubation.promotion.scenario.deferred]
- GIVEN another product has memory, row, byte, concurrency, quota, or timeout limits but no durable reservation and settlement lifecycle
- WHEN promotion is evaluated
- THEN external extraction MUST remain deferred

#### Scenario: Two-consumer promotion becomes eligible
r[lattice.budget_ledger_incubation.promotion.scenario.eligible]
- GIVEN a second consumer replaces material duplicate reserve-and-settle decisions and both adapters pass positive and negative parity
- WHEN a separate promotion change is reviewed
- THEN it MAY propose a licensed independent repository with a canonical Radicle RID, exact immutable revisions, and no GitHub or sibling fallback

#### Scenario: Accounting checks are overclaimed
r[lattice.budget_ledger_incubation.promotion.scenario.non-claim]
- GIVEN local ledger and integration tests pass
- WHEN evidence is reported
- THEN it MUST NOT claim provider usage truth, billing truth, elapsed time, resource availability, fairness, work completion, policy correctness, or release readiness
