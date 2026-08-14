## Phase 1: Baseline, references, and ownership

- [x] [serial] B1 Run and record current agent runtime, workflow, orchestration, budget, recovery, artifact, redaction, Animus leaf, and operator harness baselines. r[lattice.agent_evaluation.verification]
- [x] [serial] B2 Inventory current case-like fixtures, examples, live smokes, verifier paths, artifact stores, result reports, and model routes. r[lattice.agent_evaluation.catalog] r[lattice.agent_evaluation.execution]
- [x] [parallel] B3 Record the exact camelAI reference revision, MIT provenance, case classes, standard envelope mechanics, adopt and reject decisions, and no-runtime-dependency boundary. r[lattice.agent_evaluation.catalog] r[lattice.agent_evaluation.evidence]
- [x] [parallel] B4 Map Animus Harbor, trajectory, role, receipt, and completion boundaries to Lattice-owned execution without ambient sibling paths. r[lattice.agent_evaluation.execution] r[lattice.agent_evaluation.evidence]

## Phase 2: Typed catalog and pure planning

- [x] [serial] C1 Add typed Nickel contracts for catalog, case classes, subjects, workload revisions, agent and environment profiles, budgets, gates, rubrics, references, artifacts, live posture, retention, and non-claims. r[lattice.agent_evaluation.catalog] r[lattice.agent_evaluation.case_contract]
- [x] [serial] C2 Export deterministic runtime data and add strict Rust revalidation with duplicate, drift, unsafe-default, contradiction, and unknown-field rejection. r[lattice.agent_evaluation.catalog]
- [x] [serial] C3 Add domain-separated BLAKE3 catalog, case revision, subject, plan, trial, gate-set, rubric, reference-set, artifact-policy, deployment, and receipt identities. r[lattice.agent_evaluation.case_contract] r[lattice.agent_evaluation.planning]
- [x] [serial] C4 Add pure case-matrix planning over explicit selections, subjects, routes, environments, authorities, finite budgets, and resume facts. r[lattice.agent_evaluation.planning]
- [x] [serial] C5 Add exact resume admission and deterministic plan-order result rendering. r[lattice.agent_evaluation.planning]
- [x] [parallel] C6 Add catalog and asset parity validation for missing, undeclared, duplicate, stale, or multiply owned case assets. r[lattice.agent_evaluation.catalog] r[lattice.agent_evaluation.verification]

## Phase 3: Case and oracle contracts

- [x] [serial] O1 Add pure validated `unit`, `skill`, `hard`, and `live` case contracts with separate difficulty and effect posture. r[lattice.agent_evaluation.case_contract]
- [x] [serial] O2 Add immutable subject and mutation-scope validation for workload, fixture, scaffold, initial-state, hidden-material, and evaluator identities. r[lattice.agent_evaluation.case_contract]
- [x] [serial] O3 Add deterministic hard-gate definitions and evidence reduction for persisted state, files, APIs, arithmetic truth, safety, harness integrity, artifacts, redaction, and forbidden effects. r[lattice.agent_evaluation.oracles]
- [x] [serial] O4 Add frozen reference-evidence and freshness validation that prevents agent creation, mutation, selection, or approval of its own ground truth. r[lattice.agent_evaluation.oracles]
- [x] [serial] O5 Add separate advisory rubric-judge input and output contracts that cannot override hard gates, authority, safety, budgets, or Cairn completion. r[lattice.agent_evaluation.oracles]

## Phase 4: Injected runner and live posture

- [x] [serial] H1 Add implementation-free workspace, agent, workflow, deployment, deterministic verifier, judge, artifact, cleanup, and clock service ports. r[lattice.agent_evaluation.execution]
- [x] [serial] H2 Add a thin host runner that persists durable trial start before agent or live effects and returns typed service evidence. r[lattice.agent_evaluation.execution]
- [x] [serial] H3 Reuse existing bounded-exec, workflow revision, orchestration, budget, cancellation, recovery, and unknown-outcome owners without hidden fallback. r[lattice.agent_evaluation.execution] r[lattice.agent_evaluation.planning]
- [x] [serial] L1 Add live-case admission for explicit operator authority, isolated target, opaque credentials, deployment count, retention, cleanup, and unknown-outcome policy. r[lattice.agent_evaluation.live]
- [x] [serial] L2 Add live recovery that distinguishes started, deployed, validated, partial, failed, cancelled, cleanup-required, and unknown outcomes without automatic retry or promotion. r[lattice.agent_evaluation.live]
- [x] [serial] L3 Add selected bounded live smoke profiles that use isolated test targets and cannot mutate production or release state. r[lattice.agent_evaluation.live] r[lattice.agent_evaluation.verification]

## Phase 5: Artifacts and evidence

- [x] [serial] E1 Add artifact policies for roles, media, byte and count bounds, safe relative destinations, redaction, retention, and public or private classes. r[lattice.agent_evaluation.evidence]
- [x] [serial] E2 Add artifact admission and manifest projection with BLAKE3 identities, collision checks, and symlink, absolute, remote, secret, hidden-material, and oversized rejection. r[lattice.agent_evaluation.evidence]
- [x] [serial] E3 Add one standard metadata-only trial envelope with bounded outcome classes, counts, artifact references, redaction, non-claims, and receipt identity. r[lattice.agent_evaluation.evidence]
- [x] [serial] E4 Add optional import of validated Animus Harbor or trajectory facts through pinned adapters without raw transcript receipt expansion. r[lattice.agent_evaluation.evidence]

## Phase 6: Positive and negative validation

- [x] [parallel] T1 Add positive unit and skill cases for one mechanism, multi-step user capability, deterministic persisted state, artifact production, and advisory rubric output. r[lattice.agent_evaluation.verification]
- [x] [parallel] T2 Add positive hard cases with frozen arithmetic or API ground truth, adversarial safety gates, exact artifacts, and no live authority by default. r[lattice.agent_evaluation.oracles] r[lattice.agent_evaluation.verification]
- [x] [parallel] T3 Add positive fake-live and selected isolated-live cases for deploy, validation, retention, cleanup, and explicit unknown outcomes. r[lattice.agent_evaluation.live] r[lattice.agent_evaluation.verification]
- [x] [parallel] T4 Add negative catalog, subject, budget, route, environment, gate, rubric, reference, artifact, resume, cancellation, and stale-revision cases. r[lattice.agent_evaluation.catalog] r[lattice.agent_evaluation.planning] r[lattice.agent_evaluation.verification]
- [x] [parallel] T5 Add negative oracle cases for agent-authored ground truth, edited gates, judge override, incidental-action hard gates, missing evidence, malformed evidence, timeout, and contradictory results. r[lattice.agent_evaluation.oracles] r[lattice.agent_evaluation.verification]
- [x] [parallel] T6 Add negative live cases for missing authority, production targets, credential leakage, target drift, duplicate deployment, uncertain outcome, cleanup failure, auto-retry, and automatic promotion. r[lattice.agent_evaluation.live] r[lattice.agent_evaluation.verification]
- [x] [parallel] T7 Add receipt and artifact leak tests for prompts, transcripts, reasoning, tool bodies, provider data, credentials, paths, hidden tests, raw logs, judge rationale, and unsupported quality claims. r[lattice.agent_evaluation.evidence] r[lattice.agent_evaluation.verification]

## Phase 7: Product surface and closeout

- [x] [depends:add-guarded-agent-loop-templates] D1 Add reviewed agent-evaluation workflow or orchestration bundles after the guarded loop boundary is available. r[lattice.agent_evaluation.execution]
- [x] [serial] D2 Add headless catalog list, plan, run, resume, inspect, and artifact commands with preview-first live behavior and safe JSON output. r[lattice.agent_evaluation.catalog] r[lattice.agent_evaluation.execution] r[lattice.agent_evaluation.evidence]
- [x] [serial] D3 Document catalog authoring, case classes, hard gates, rubrics, references, live authority, artifacts, resume, operator recovery, and non-claims. r[lattice.agent_evaluation.case_contract] r[lattice.agent_evaluation.evidence]
- [x] [serial] D4 Update README, examples, repository validation, traceability, operator diagnostics, and reference entries. r[lattice.agent_evaluation.catalog] r[lattice.agent_evaluation.verification]
- [x] [serial] D5 Add a focused evaluation rail and Nix check for Nickel drift, catalog parity, pure planning, injected services, oracles, live fakes, artifacts, receipts, restart behavior, and dependency isolation. r[lattice.agent_evaluation.verification]
- [x] [serial] D6 Run focused tests, formatting, Clippy, `scripts/check.rs`, Octet, Cairn validation and gates, and bounded `nix flake check -L --option secret-key-files ''`. r[lattice.agent_evaluation.verification]
- [x] [serial] D7 Review dry-run sync and archive plans, make sure that every requirement reaches the accepted spec, and archive only after current positive and negative evidence passes. r[lattice.agent_evaluation.verification]
