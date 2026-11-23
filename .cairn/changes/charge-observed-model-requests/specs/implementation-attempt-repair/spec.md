# Implementation Attempt Repair And Budget Lineage

## ADDED Requirements

### Requirement: Observed model requests are charged exactly
r[lattice.implementation_attempt.observed_charge] When an attempt's retained broker evidence is complete, Lattice MUST charge its lineage exactly the model requests that evidence proves dispatched, and MUST report that count with `host_measured` provenance. Complete means every intent in scope has a terminal record, none ended `uncertain` and the evidence does not conflict. A host-observed count is not a worker report, so it MAY charge less than one broker ceiling. In every other case the charge MUST stay the reservation, one broker ceiling, with a `null` count and `unknown` provenance. A retained charge MUST never be lowered.

#### Scenario: A complete single-shot run is charged its dispatched requests
r[lattice.implementation_attempt.observed_charge.scenario.single_shot]
- GIVEN a single-shot attempt with a lineage whose broker evidence is complete, with or without a serving declaration
- WHEN its result is published
- THEN its `budget_lineage` usage MUST report `model_requests` equal to the dispatched requests with provenance `host_measured`, and charge exactly that count
- AND a served attempt's coverage MUST keep its service attribution

#### Scenario: Incomplete evidence keeps the reservation
r[lattice.implementation_attempt.observed_charge.scenario.incomplete]
- GIVEN an intent without a terminal record, an `uncertain` request, conflicting evidence or no retained evidence in the scope of a single shot or a repair step
- WHEN it is charged
- THEN its usage MUST report `model_requests: null` with provenance `unknown` and charge the reservation, and conflicting evidence MUST fail closed before any usage or draw is retained

#### Scenario: Repair steps count their own index range
r[lattice.implementation_attempt.observed_charge.scenario.step_range]
- GIVEN a repair run whose steps relay requests through the attempt broker
- WHEN step `n` is charged
- THEN it MUST count only the evidence with request indices in `[(n - 1) * max_requests, n * max_requests)`, so a request or an incomplete intent of another step never changes its count

#### Scenario: Recovery replays retained draws
r[lattice.implementation_attempt.observed_charge.scenario.replay]
- GIVEN an attempt whose draw was retained, a single shot's write-once `lineage-draw.json` before its result or a repair step's charge in its disposition
- WHEN recovery, a duplicate run or inspection replays it, even after its request evidence was lost or changed
- THEN it MUST report the retained draw unchanged, a different draw MUST NOT replace it, and an attempt without a retained single-shot draw MUST report the reservation, never a lower count derived afterwards
