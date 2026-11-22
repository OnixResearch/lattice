# Implementation Attempt Repair Export

## ADDED Requirements

### Requirement: Repaired candidates are exported and inventoried from their retained disposition

r[lattice.implementation_attempt.repair_export] `export-candidate` of an attempt with a repair policy MUST export only the exact final candidate revision named by its retained passing disposition, after the same request, intent, output and revision-seam checks as a single-shot export, and `inspect` MUST report a retained disposition within `lattice.worker-inventory.v3`. Neither MUST guess an outcome that is not retained, and single-shot attempts MUST behave as before.

#### Scenario: A passing disposition exports its exact revision

r[lattice.implementation_attempt.repair_export.scenario.passing]
- GIVEN a retained `correct_candidate` or `repaired_candidate` disposition of the exact request
- WHEN `export-candidate` runs with a permitted or unmanaged revision
- THEN the named revision MUST be the last recorded revision, at iteration 1 for `correct_candidate` and `repairs + 1` for `repaired_candidate`, equal to the recorded step count
- AND every recorded revision MUST re-derive from this envelope, attempt and fence, its iteration, its parent (the revision before it) and its receipt, the record MUST name this envelope and execution binding, the final receipt MUST be a produced receipt of this work, attempt, fence, workspace and base, and the step record of the final iteration MUST record the same revision with a `passed` verdict
- AND the workspace MUST capture to that receipt's candidate identity, and the export MUST contain that snapshot, the final revision's receipt as `host-receipt.json` and the retained disposition as `native-observation.json`, replayed exactly, with `accepted:false` and `integrated:false`

#### Scenario: Anything else is refused and nothing is exported

r[lattice.implementation_attempt.repair_export.scenario.refused]
- GIVEN an attempt with a repair policy whose disposition is missing, malformed, of other request bytes, contradicted by its records or this request, or not passing, or whose managed revision was revoked, or whose workspace no longer captures to the final receipt
- WHEN `export-candidate` runs
- THEN it MUST export nothing and refuse a missing disposition as `repair_outcome_unknown`, a non-passing one as `repair_not_candidate`, a malformed, crossed or contradicted one as `repair_disposition_invalid`, and a revoked revision as `export_revoked`, each with `dispatched:true`
- AND a crash between a step record and the disposition MUST stay `repair_outcome_unknown`, never inferred from step records

#### Scenario: Inventory reports the retained disposition

r[lattice.implementation_attempt.repair_export.scenario.inventory]
- GIVEN an attempt with a repair policy and an execution intent
- WHEN `inspect` runs
- THEN a retained disposition of the exact request MUST be `terminal_observed` with the disposition's `kind`, `exited` liveness and no wait reasons, and `cancelled` MUST be observed termination
- AND without a retained disposition the instance MUST stay `dispatch_recorded_outcome_unknown` with the `dispatch_outcome_unknown` reason, and a malformed or crossed disposition MUST be an error
- AND the document MUST keep the `lattice.worker-inventory.v3` schema and enumerations
