# Implementation Attempt Repair Export

## ADDED Requirements

### Requirement: Repaired candidates are exported and inventoried from their complete retained records

r[lattice.implementation_attempt.repair_export] `export-candidate` of an attempt with a repair policy MUST export only the exact final candidate revision named by its retained passing disposition, after the same request, intent, output and revision-seam checks as a single-shot export, and `inspect` MUST report a retained disposition within `lattice.worker-inventory.v3`. Both MUST act only on the complete typed record set of the exact request, the disposition and the intent and record of every step it counts, re-derived and cross-checked; no single recorded field, such as a kind, a verdict or a receipt's own claims, MUST be trusted alone, no outcome that is not retained MUST be guessed, and single-shot attempts MUST behave as before.

#### Scenario: A passing disposition exports its exact revision

r[lattice.implementation_attempt.repair_export.scenario.passing]
- GIVEN a retained v2 `correct_candidate` or `repaired_candidate` disposition naming this request's bytes, envelope, persisted execution binding, policy, lineage and dispatch plan, with complete typed step intents and records
- WHEN `export-candidate` runs with a permitted or unmanaged revision
- THEN the repairs MUST fit the admitted policy's iterations and the global bound, the step count MUST be `repairs + 1`, every step MUST be this attempt's exited step with the parent and failure classes the loop would have given it, and the named revision MUST be the last recorded one
- AND every recorded revision MUST re-derive from this envelope, attempt and fence, its iteration, its parent, its candidate tree, execute bits and diff, and carry a produced receipt of this work, attempt, fence, workspace and base whose changed paths are exactly its retained base digests
- AND every step's verdict MUST be re-derived from its check observations and the integrity of each declared check's copy and equal the recorded one; each step before the last MUST fail only with admitted classes and the last MUST pass
- AND the workspace MUST keep the retained execute bits, and the whole receipt MUST re-derive from the captured workspace and the retained base digests of the changed paths, with the admitted base identity
- AND the export MUST contain that snapshot, the receipt as `host-receipt.json` and the retained disposition as `native-observation.json`, replayed exactly, with `accepted:false` and `integrated:false`

#### Scenario: Anything else is refused and nothing is exported

r[lattice.implementation_attempt.repair_export.scenario.refused]
- GIVEN an attempt with a repair policy whose disposition is missing, malformed, incomplete, of the first schema, of other request bytes or another envelope or execution binding, contradicted by its records or this request, beyond the admitted iterations, relabelled from a failed or drifted check, or not passing, or whose managed revision was revoked, or whose workspace bytes, execute bits or changed-file manifest no longer re-derive the final receipt
- WHEN `export-candidate` runs
- THEN it MUST export nothing and refuse a missing disposition as `repair_outcome_unknown`, a non-passing one as `repair_not_candidate`, any other unexportable record set as `repair_disposition_invalid`, and a revoked revision as `export_revoked`, each with `dispatched:true`
- AND a crash between a step record and the disposition MUST stay `repair_outcome_unknown`, never inferred from step records

#### Scenario: Inventory reports only a proven stop

r[lattice.implementation_attempt.repair_export.scenario.inventory]
- GIVEN an attempt with a repair policy and an execution intent
- WHEN `inspect` runs
- THEN the instance MUST be `terminal_observed` with the disposition's `kind`, `exited` liveness and no wait reasons only when the complete record set agrees with the kind and every recorded step reports its worker exited or cancelled, and `cancelled` MUST be observed termination
- AND `uncertain` with cause `worker_outcome`, a step intent without a record other than the one `budget_exhausted` (`elapsed`) or `uncertain` (`clock`) leaves before its worker is called, a missing disposition and a first-schema disposition MUST stay `dispatch_recorded_outcome_unknown` with the `dispatch_outcome_unknown` reason
- AND a malformed, incomplete, crossed or contradicted record set MUST be an error, never terminal
- AND the document MUST keep the `lattice.worker-inventory.v3` schema and enumerations
