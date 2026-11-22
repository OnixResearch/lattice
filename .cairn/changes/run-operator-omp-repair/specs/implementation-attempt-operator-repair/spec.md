# Implementation Attempt Operator Repair

## ADDED Requirements

### Requirement: The standalone CLI runs bounded repair through operator-authorized OMP and catalog ports

r[lattice.implementation_attempt.operator_repair] `lattice implementation-attempt run` of a request with a repair policy and without embedder repair ports MUST run the bounded same-attempt repair loop through the standalone operator ports only: one pinned OMP step per iteration through the single-shot attempt's workspace boundary and OMP launcher, and every declared check as one bounded process of the operator's `--repair-checks` catalog through the candidate worker's generic bubblewrap route on the check's read-only copy of the candidate. Each step MUST keep the loop's attempt, fence, workspace and budget lineage and its step intents and records. The ports MUST be used only while the operator's `effect_authority` profile grants the exact worker and check effects of the request's worker profile revision; they MUST grant nothing themselves and MUST NOT be used for a request without a repair policy.

#### Scenario: A failing check is repaired by a second OMP step

r[lattice.implementation_attempt.operator_repair.scenario.repaired]
- GIVEN a request with a repair policy naming the catalog's executor and checks, a pinned catalog and a profile granting the worker and check tuples of its worker profile revision
- WHEN the first OMP step's candidate fails a check with an admitted class and the second step's candidate passes
- THEN the disposition MUST be `repaired_candidate` with one repair, the second step MUST start from the parent revision with a prompt naming only that revision and the admitted classes, and each launched step MUST retain its raw events and launcher observation before its step record
- AND recovery and a duplicate `run` MUST launch nothing, inventory MUST report `terminal_observed`, and `export-candidate` MUST export the candidate only through the existing re-derivation checks

#### Scenario: Without the exact grant nothing is launched

r[lattice.implementation_attempt.operator_repair.scenario.authority]
- GIVEN a repair request whose configuration selects no profile, or whose profile is expired, names another worker profile revision or lacks the worker or check tuple
- WHEN `run` admits it
- THEN it MUST be refused as `repair_authority_denied` with `dispatched:false` before the fence record, writing only the attempt-fence's closure marker and no step intent, and no OMP or check process MUST start
- AND a profile that stops granting, changes identity, or an admission that expires during the run MUST end the loop as `authority_withdrawn` with cause `effect_authority` and its recheck evidence when found before a step's intent or before a revision's checks, and MUST leave the later checks unrun and the loop `uncertain` when found between individual checks, so a candidate checked after the grant or admission lapsed is never passed or exported; a cancellation requested during the checks MUST end the loop as `cancelled`

#### Scenario: Worker outcomes keep the loop's classes

r[lattice.implementation_attempt.operator_repair.scenario.outcomes]
- GIVEN a launched OMP step
- WHEN it exits, dies by a signal inside the sandbox, is stopped at its deadline or output limit, is cancelled through a persisted request, is killed without an exit status, is blocked before spawn (including a workspace that is not the tree the previous step left), or ends with an incomplete terminal stream or an unknown teardown
- THEN an observed exit, a signal death reported as `128 + N`, and a host stop at an uncapped deadline or the output limit MUST be `exited`, cancellation MUST be `cancelled`, and every other outcome, including a stop at a deadline the admission's expiry capped, MUST be `unknown`, so the loop ends `uncertain` without retrying or checking it
- AND a worker that runs to an uncapped deadline MUST end the loop as `budget_exhausted` (`elapsed`) unchecked, the iteration bound MUST stop further steps, a newer fence MUST end it as `fence_superseded`, and inventory MUST report only observed stops

#### Scenario: Catalog checks are pinned, bounded and isolated

r[lattice.implementation_attempt.operator_repair.scenario.checks]
- GIVEN an operator check catalog
- WHEN `run` admits a repair request
- THEN a missing catalog MUST keep the `repair_ports_unavailable` refusal, and an unreadable, oversized, malformed, unpinned, byte-mismatched catalog, one inside the attempt workspace, or one whose executor and check configurations differ from the policy's MUST be refused as `repair_ports_unavailable` with its `check_catalog` reason before any launch
- AND each check MUST run with a cleared environment, a private network, no broker or credential, no write path, bounded output and a deadline within the admission's expiry, only on the request's read paths present in its copy, and only an exit observed on an unchanged copy MUST decide it: a status from 1 to 125 fails with the entry's class, and a cannot-execute, not-found or signal status is unknown

#### Scenario: A controller lost after a step intent never relaunches

r[lattice.implementation_attempt.operator_repair.scenario.recovery]
- GIVEN an operator repair run lost after a step intent and before its record
- WHEN `recover` or a duplicate `run` reads the attempt
- THEN the outcome MUST stay unknown, inventory MUST stay `dispatch_recorded_outcome_unknown`, and no OMP step MUST launch again

#### Scenario: Requests without a repair policy are unchanged

r[lattice.implementation_attempt.operator_repair.scenario.absent]
- GIVEN a request without a repair policy
- WHEN the standalone CLI runs it with its operator ports
- THEN it MUST keep its bytes, read no configuration or catalog, never use the operator launcher, and retain the same attempt records as a run without operator ports
