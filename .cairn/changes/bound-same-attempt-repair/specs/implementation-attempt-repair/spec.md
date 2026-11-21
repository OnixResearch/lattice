# Implementation Attempt Repair And Budget Lineage

## ADDED Requirements

### Requirement: Bounded same-attempt repair

r[lattice.implementation_attempt.repair] An implementation attempt MAY carry an admitted repair policy. With one, every worker step of the attempt MUST be decided by a pure bounded loop, every candidate MUST be derived by the host from the attempt workspace, recorded as a new revision bound to the same attempt and fence and checked by every declared check through the injected host check port, and the attempt MUST end in exactly one retained disposition. No disposition is acceptance: independent acceptance stays with Kiln #46.

#### Scenario: The policy is admitted and framed

r[lattice.implementation_attempt.repair.scenario.policy]
- GIVEN a request with a repair policy
- WHEN it is validated
- THEN its failure classes and checks MUST be strictly sorted, bounded and well formed, its check executor and configurations MUST be BLAKE3 identities, its repair iterations plus the first step MUST fit the envelope's turns, its elapsed bound MUST fit the envelope's seconds, and its aggregate budgets MUST cover one step and its checks
- AND a policy combined with `allow_unchanged` or served model routes MUST be refused
- AND the profile revision MUST be the single-shot revision bound to the policy identity, so a changed policy is a new revision

#### Scenario: Distinct dispositions

r[lattice.implementation_attempt.repair.scenario.dispositions]
- GIVEN a repair loop
- WHEN the first candidate passes, a repaired candidate passes, an iteration, elapsed, model, tool or lineage bound would be exceeded by the next step, cancellation is requested, a worker or check outcome is unknown, or a failure lies outside the admitted classes
- THEN the disposition MUST be `correct_candidate`, `repaired_candidate`, `budget_exhausted` with its bound, `cancelled`, `uncertain` with its cause, or `not_repairable`, respectively, and no further worker step MUST spawn
- AND every step MUST persist its intent before its worker effect, and recovery MUST report only the retained disposition, never spawn, check or charge again, and treat a missing disposition as an unknown outcome

#### Scenario: Checks cannot be bypassed

r[lattice.implementation_attempt.repair.scenario.bypass]
- GIVEN a step that exits zero, produces no new candidate, writes its own success claims, changes a check's configuration, or whose check is skipped, missing, duplicated, undeclared, run against another revision, run by another executor or run by the implementing worker profile itself
- WHEN its checks are classified
- THEN the candidate MUST NOT pass: no new candidate is the failure class `no_candidate`, a failing check is its failure class, and every other case MUST reject the evidence as `checks_rejected` with its defect

#### Scenario: Changed work stays with Kiln

r[lattice.implementation_attempt.repair.scenario.kiln_owned]
- GIVEN a step whose binding names other work, another work version or task, or another attempt, fence or envelope, or a worker that proposes changed work or a new semantic attempt
- WHEN the loop records the step or its report
- THEN it MUST refuse the step as `changed_work` or `new_semantic_attempt`, or end as `kiln_owned` without another step or check

#### Scenario: No policy behaves as before

r[lattice.implementation_attempt.repair.scenario.absent]
- GIVEN a request without a repair policy and without a budget lineage
- WHEN it is encoded, framed and run
- THEN its bytes, request identity and profile revision MUST be unchanged and it MUST run the single bounded worker step as before
- AND a request with a repair policy MUST be refused before any write as `repair_ports_unavailable` when the embedder supplies no worker and check ports, which the standalone CLI never does

### Requirement: Kiln budget lineage consumer

r[lattice.implementation_attempt.lineage] An attempt MAY carry Kiln #61's admitted budget lineage for its account as the `lattice.kiln-budget-lineage.v1` consumer type: root, account, parent, depth, remaining amounts by dimension, lineage blocker and serving gate. Lattice MUST only draw from it: it never mints, refunds or resets quota, and it records the exact Kiln source revision and field mapping.

#### Scenario: Exhaustion refuses before spawn

r[lattice.implementation_attempt.lineage.scenario.exhausted]
- GIVEN a lineage with a blocker, a serving gate other than `none` or `eligible`, a zero remainder in any dimension, or a remainder that cannot cover the next step's reservation in a dimension Lattice charges
- WHEN an attempt is run, or its loop asks for another step
- THEN the first step MUST be refused before any write as `lineage_blocked` or `lineage_exhausted` with `dispatched:false`, and a later step MUST NOT spawn and the loop MUST end as `budget_exhausted` with the lineage dimension or `lineage_blocked`

#### Scenario: One aggregate budget

r[lattice.implementation_attempt.lineage.scenario.shared]
- GIVEN an attempt account and a child account
- WHEN the child is admitted
- THEN it MUST be this account's direct child in the same root and its ceiling in every one of this account's dimensions MUST fit what is left after this account's draws; omitting a dimension, adding one or exceeding the remainder MUST be refused as a fresh budget

#### Scenario: Usage is attributed and unknown stays unknown

r[lattice.implementation_attempt.lineage.scenario.usage]
- GIVEN preparation, edit, check and repair phases, including failed, cancelled and uncertain steps
- WHEN they are charged
- THEN each phase MUST be recorded with its reported counts and existing usage provenance, an unknown count MUST stay unknown and be charged at its reservation, a lower report MUST NOT lower the charge, and an overrun MUST be charged
- AND the attempt's `Scope::Attempt` usage observation MUST be persisted before its disposition, and dimensions Lattice cannot charge MUST be reported as unaccounted
