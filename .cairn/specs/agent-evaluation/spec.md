# Agent Evaluation Specification

## Purpose

Defines the `agent-evaluation` capability.

## Requirements

### Requirement: One typed catalog owns committed evaluation discovery

r[lattice.agent_evaluation.catalog] Lattice MUST use one typed Nickel catalog as the authoritative source for committed agent-evaluation case discovery. Rust MUST revalidate generated catalog data before planning or execution.

#### Scenario: Valid catalog is admitted

r[lattice.agent_evaluation.catalog.valid]
- GIVEN generated catalog data has unique case IDs, immutable revisions, complete typed fields, finite bounds, declared assets, and required non-claims
- WHEN Rust catalog validation runs
- THEN Lattice MUST return deterministic admitted cases and a domain-separated BLAKE3 catalog identity
- AND committed case discovery MUST match the exact declared case assets.

#### Scenario: Catalog or assets drift

r[lattice.agent_evaluation.catalog.drift]
- GIVEN catalog data is stale, malformed, duplicated, unbounded, contradictory, or differs from declared case assets
- WHEN catalog or parity validation runs
- THEN Lattice MUST fail closed with bounded diagnostics
- AND it MUST plan or execute no agent, workflow, verifier, judge, deployment, artifact, cleanup, or promotion effect.

### Requirement: Evaluation cases bind exact subjects and policies

r[lattice.agent_evaluation.case_contract] Every evaluation case MUST bind one class, immutable subject, workload revision, mutation scope, agent profile, environment, budgets, gate set, rubric, reference set, artifact policy, live posture, retention policy, and non-claim set.

#### Scenario: Unit or skill case is complete

r[lattice.agent_evaluation.case_contract.local]
- GIVEN a `unit` or `skill` case declares one exact subject, local or isolated environment, positive budgets, deterministic gates, optional rubric, references, artifacts, and no hidden live effects
- WHEN case validation runs
- THEN Lattice MUST return one deterministic case revision identity and effect posture
- AND undeclared network, deployment, credentials, fallback, or mutation authority MUST remain denied.

#### Scenario: Hard and live posture remain distinct

r[lattice.agent_evaluation.case_contract.hard_live]
- GIVEN a `hard` case requires stronger ground truth or adversarial gates and also requests live effects
- WHEN case validation runs
- THEN it MUST satisfy both the hard-case contract and every live admission requirement
- AND difficulty alone MUST NOT grant network, deployment, credential, retention, or cleanup authority.

#### Scenario: Case subject or evaluator drifts

r[lattice.agent_evaluation.case_contract.drift]
- GIVEN workload, initial state, fixture, scaffold, mutation scope, hidden-material posture, agent profile, environment, gate, rubric, reference, artifact, or live policy differs from the bound case revision
- WHEN trial admission or result validation runs
- THEN Lattice MUST reject the stale case or result
- AND it MUST NOT compare or resume the changed trial as the original case revision.

### Requirement: Evaluation matrix planning is pure and exact

r[lattice.agent_evaluation.planning] Lattice MUST plan evaluation matrices and resume decisions through pure deterministic logic over admitted cases, explicit selections, subjects, routes, environments, authority, finite budgets, and prior result metadata.

#### Scenario: Exact matrix plans in stable order

r[lattice.agent_evaluation.planning.valid]
- GIVEN selected cases, agent profiles, environments, trial counts, concurrency limits, subjects, and budgets all pass admission
- WHEN pure matrix planning runs
- THEN Lattice MUST emit ordered trial plans with exact BLAKE3 plan and trial identities
- AND planning MUST perform no filesystem, process, provider, network, deployment, clock, verifier, artifact, or cleanup effect.

#### Scenario: Prior trial qualifies for resume

r[lattice.agent_evaluation.planning.resume]
- GIVEN a prior terminal trial exactly matches case revision, subject, agent, environment, policy, gates, rubric, references, artifacts, status, and expected receipt identity
- WHEN resume admission runs
- THEN Lattice MAY skip that exact trial and retain deterministic plan-order output
- AND partial, stale, failed, cancelled, cleanup-required, live-unknown, or artifact-mismatched trials MUST NOT become complete through resume.

#### Scenario: Required service or budget is unavailable

r[lattice.agent_evaluation.planning.blocked]
- GIVEN a required service, authority, environment, credential handle, or positive budget is missing, unavailable, exhausted, stale, or contradictory
- WHEN trial planning runs
- THEN Lattice MUST block before dependent effects
- AND it MUST NOT select a fallback agent, provider, environment, target, verifier, judge, artifact store, or cleanup service.

### Requirement: Evaluation execution uses injected bounded services

r[lattice.agent_evaluation.execution] Lattice MUST execute admitted trials through explicit injected workspace, agent, workflow, verifier, judge, deployment, artifact, cleanup, and clock services. Reusable decision code MUST remain effect-free.

#### Scenario: Admitted local trial runs

r[lattice.agent_evaluation.execution.local]
- GIVEN a local trial plan has matching pre-state, available services, admitted authority, sufficient budget, and one durable trial-start record
- WHEN the host runner executes the plan
- THEN it MAY prepare the workspace, run the exact agent or workflow, capture post-state, run validations, admit artifacts, and persist one result
- AND every service outcome MUST remain correlated to the exact trial and plan identities.

#### Scenario: Subject changes after planning

r[lattice.agent_evaluation.execution.subject_changed]
- GIVEN observed workspace or workload state no longer matches the planned subject before agent or verifier dispatch
- WHEN the shell rechecks the trial
- THEN it MUST block the stale plan before the dependent effect
- AND it MUST NOT repair, reset, replace, or widen the subject automatically.

#### Scenario: Execution ends ambiguously

r[lattice.agent_evaluation.execution.ambiguous]
- GIVEN an agent, workflow, verifier, deployment, artifact, or cleanup service starts but no exact terminal observation is available
- WHEN recovery evaluates the trial
- THEN Lattice MUST retain a pending or unknown outcome according to case policy
- AND it MUST NOT invent success, failure, cancellation, cleanup, score, completion, or retry authority.

### Requirement: Deterministic hard gates remain final

r[lattice.agent_evaluation.oracles] Lattice MUST keep deterministic state, behavior, safety, harness, artifact, and redaction validations as hard gates. Semantic rubric judgment MUST remain separate and advisory.

#### Scenario: Hard gates and rubric both pass

r[lattice.agent_evaluation.oracles.pass]
- GIVEN every required current hard gate passes over the exact subject and artifacts and an optional rubric judge returns an admitted score
- WHEN pure result reduction runs
- THEN Lattice MAY classify the trial with both deterministic and advisory outcomes
- AND the advisory outcome MUST retain judge, rubric, model, subject, and policy attribution.

#### Scenario: Judge passes while a hard gate fails

r[lattice.agent_evaluation.oracles.hard_failure]
- GIVEN a judge returns a passing score but one required hard gate is failed, missing, stale, timed out, malformed, or contradictory
- WHEN result reduction runs
- THEN the trial MUST remain non-passing under the hard-gate contract
- AND judge output MUST NOT override the gate, grant authority, suppress diagnostics, authorize promotion, or establish completion.

#### Scenario: Agent attempts to control ground truth

r[lattice.agent_evaluation.oracles.ground_truth]
- GIVEN an agent creates, edits, selects, replaces, or approves the rubric, hard-gate implementation, frozen reference evidence, or hidden materials used for its trial
- WHEN evaluator-integrity validation runs
- THEN Lattice MUST reject the affected evidence or trial before result acceptance
- AND reference evidence MUST remain immutable, BLAKE3-identified, freshness-checked, and outside agent mutation authority.

### Requirement: Live cases require separate authority and recovery

r[lattice.agent_evaluation.live] A live trial MUST require explicit operator authority, an isolated target, opaque credential handles, finite live-effect bounds, retention, cleanup, and unknown-outcome policy before any external effect.

#### Scenario: Live trial is admitted

r[lattice.agent_evaluation.live.admitted]
- GIVEN a live posture names an authorized operator, isolated target, exact environment, credential handles, deployment count, retention, cleanup, recovery, and finite budgets
- WHEN live preflight and reservation pass
- THEN Lattice MAY persist durable trial-start state and invoke the exact admitted live service
- AND the live grant MUST NOT authorize production targets, model promotion, workflow promotion, release mutation, commit, push, or pull-request operations.

#### Scenario: Live authority or target is unsafe

r[lattice.agent_evaluation.live.denied]
- GIVEN live authority is missing, stale, broad, self-granted, production-scoped, target-drifted, credential-bearing, unbounded, or lacks retention and unknown-outcome policy
- WHEN live admission runs
- THEN Lattice MUST reject the trial before external effects
- AND it MUST NOT select another target, credential, provider, namespace, or cleanup policy.

#### Scenario: Live effect outcome is unknown

r[lattice.agent_evaluation.live.unknown]
- GIVEN a live effect started and recovery cannot correlate an exact terminal deployment, validation, or cleanup observation
- WHEN trial recovery runs
- THEN Lattice MUST record an explicit unknown or cleanup-required outcome
- AND it MUST NOT retry automatically, claim deployment success, promote the result, or erase the retained uncertainty.

### Requirement: Evaluation artifacts and receipts stay bounded

r[lattice.agent_evaluation.evidence] Lattice MUST keep public evaluation receipts metadata-only. Detailed artifacts MUST remain separately authorized, allowlisted, bounded, redacted, identity-checked, and referenced through safe manifests.

#### Scenario: Safe trial result is persisted

r[lattice.agent_evaluation.evidence.safe]
- GIVEN trial execution, hard gates, optional judgment, artifacts, live observations, and cleanup evidence pass their exact validations
- WHEN result and receipt projection run
- THEN Lattice MUST emit one standard envelope with safe identities, classes, counts, outcomes, artifact-manifest reference, redaction, non-claims, and receipt identity
- AND the envelope MUST not embed private execution content.

#### Scenario: Artifact is unsafe or undeclared

r[lattice.agent_evaluation.evidence.artifact_rejected]
- GIVEN an artifact is undeclared, oversized, colliding, symlinked, absolute, remote, secret-bearing, hidden-material-bearing, malformed, or outside its retention and media policy
- WHEN artifact admission runs
- THEN Lattice MUST reject the artifact before result acceptance
- AND it MUST NOT copy, publish, hash into an unapproved receipt field, or silently omit required artifact failure.

#### Scenario: Receipt contains private data or an overclaim

r[lattice.agent_evaluation.evidence.receipt_rejected]
- GIVEN a receipt draft contains prompts, transcripts, model responses, reasoning, tool bodies, credentials, environment values, private paths, hidden tests, unrestricted logs, judge rationale, or unsupported quality claims
- WHEN receipt validation runs
- THEN it MUST reject the draft without echoing the private value
- AND the rejected receipt MUST NOT contribute to completion, promotion, lifecycle, release, or benchmark claims.

### Requirement: Agent evaluation has positive and negative product evidence

r[lattice.agent_evaluation.verification] The evaluation lane MUST include positive and negative tests through production catalog, planning, runner, oracle, live, artifact, recovery, and receipt entrypoints.

#### Scenario: Positive local and live matrix passes

r[lattice.agent_evaluation.verification.positive]
- GIVEN reviewed fixtures cover unit, skill, hard, fake-live, selected isolated-live, hard gates, advisory rubrics, frozen references, artifacts, resume, retention, cleanup, and unknown outcomes
- WHEN the focused evaluation rail runs
- THEN production entrypoints MUST return the expected deterministic plans, outcomes, identities, and ordered diagnostics
- AND live cases MUST remain limited to their exact admitted test targets.

#### Scenario: Adversarial evaluation attempt fails

r[lattice.agent_evaluation.verification.negative]
- GIVEN a fixture attempts catalog drift, evaluator mutation, judge override, live escalation, stale resume, artifact escape, secret leakage, raw transcript receipt, automatic retry, automatic promotion, or overclaiming
- WHEN the focused evaluation rail runs
- THEN the attempt MUST fail before the prohibited effect, result acceptance, or claim
- AND existing workflow, orchestration, authority, budget, recovery, artifact, Animus, Cairn, and release boundaries MUST remain intact.

#### Scenario: Change is ready to archive

r[lattice.agent_evaluation.verification.closeout]
- GIVEN maintainers intend to sync and archive the manifest-driven evaluation change
- WHEN closeout validation runs
- THEN Nickel, catalog parity, pure planning, host runner, oracle, live fake, artifact, receipt, restart, Octet, Cairn, and relevant Nix checks MUST pass
- AND agent correctness, verifier soundness, judge correctness, benchmark superiority, deployment correctness, production readiness, and release eligibility MUST remain explicit non-claims.
