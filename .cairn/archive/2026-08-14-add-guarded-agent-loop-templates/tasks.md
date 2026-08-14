## Phase 1: Dependency, references, and progress core

- [x] [serial] Complete and validate the typed UI workflow-template catalog, preview, and explicit activation boundary before implementing guarded loop content. [depends:lattice.workflow_templates.activation]
- [x] [serial] Establish and record the smallest relevant pre-change baselines for agent nodes, bounded command execution, orchestration rounds/authority/budgets/replay, workflow examples, and repository validation. r[lattice.guarded_agent_templates.rounds]
- [x] [parallel] Record the reviewed Aspen Octet and mc Ralph script identities and adopt/port/reference/reject decisions in tracked design/docs without introducing sibling runtime dependencies. r[lattice.guarded_agent_templates.evidence.scenario.non-claim]
- [x] [serial] Add the pure checked monotonic-progress policy, observation, prerequisite, decision, and diagnostic core with named positive minimum delta and no I/O. r[lattice.guarded_agent_templates.progress]
- [x] [parallel] Add positive and negative progress tests for strict decrease, minimum delta, equality, regression, missing/malformed/overflowing metrics, duplicate/conflicting observations, and failed validation/restoration prerequisites. r[lattice.guarded_agent_templates.progress.scenario.improved] r[lattice.guarded_agent_templates.progress.scenario.blocked]

## Phase 2: Cairn agent-drain bundle

- [x] [serial] Add deterministic bounded Cairn list/validation adapters and pure selection logic for explicit or sorted-first single-change admission. r[lattice.guarded_agent_templates.cairn]
- [x] [serial] Author the compact goal-first Cairn drain role prompt and one-agent acyclic workflow round with exact command/root/change/revision identities and no-push authority. r[lattice.guarded_agent_templates.rounds.scenario.one-agent] r[lattice.guarded_agent_templates.cairn]
- [x] [serial] Author the positive-budget `cairn-agent-drain` orchestration definition with empty-set completion, strict active-count reduction, inert successor proposals, and operator approval between rounds. r[lattice.guarded_agent_templates.cairn.scenario.empty] r[lattice.guarded_agent_templates.cairn.scenario.drained]
- [x] [parallel] Add positive and negative fake-adapter tests for no work/zero agent calls, one exact drained change, malformed or changed selection, unchanged/increased count, command failure/timeout/truncation, validation failure, and retained operator-owned edits. r[lattice.guarded_agent_templates.cairn.scenario.empty] r[lattice.guarded_agent_templates.cairn.scenario.drained] r[lattice.guarded_agent_templates.cairn.scenario.blocked]

## Phase 3: Octet agent burn-down bundle

- [x] [serial] Implement a bounded no-disabled probe adapter that prefers non-mutating configuration and otherwise owns exact backup/restoration checks on every observed path while blocking uncertain recovery mismatches. r[lattice.guarded_agent_templates.octet.scenario.restoration]
- [x] [serial] Author the compact goal-first Octet candidate prompt and one-agent acyclic baseline/candidate/validate/post-probe workflow with no policy relaxation, automatic rollback, commit, or push. r[lattice.guarded_agent_templates.octet]
- [x] [serial] Author the positive-budget `octet-agent-burndown` orchestration definition with strict finding-count reduction, inert successor proposals, exact restoration prerequisites, and operator approval between rounds. r[lattice.guarded_agent_templates.octet.scenario.reduced]
- [x] [parallel] Add positive and negative fake-adapter tests for strict reduction/restoration and equal/increased/missing/malformed counts, probe/validation failure/timeout/truncation, restoration failure, recovery mismatch, and retained operator-owned edits. r[lattice.guarded_agent_templates.octet.scenario.reduced] r[lattice.guarded_agent_templates.octet.scenario.restoration] r[lattice.guarded_agent_templates.octet.scenario.non-improving]

## Phase 4: Authority, UI bundles, and evidence

- [x] [serial] Enforce one-agent-per-round shape, reject hidden nested loops, bind fixed target/commands/prompts/assets/progress/budgets by identity, and route successors only through existing approval and fresh reservation admission. r[lattice.guarded_agent_templates.rounds.scenario.hidden-loop] r[lattice.guarded_agent_templates.authority]
- [x] [parallel] Add negative authority tests for second-agent, fallback, command/target/root/credential/trigger/budget mutation, self-approval, stale approval, exhausted bounds, automatic retry, commit, push, force-push, and PR requests. r[lattice.guarded_agent_templates.authority.scenario.approved] r[lattice.guarded_agent_templates.authority.scenario.denied] r[lattice.guarded_agent_templates.authority.scenario.budget]
- [x] [serial] Package both workflows, orchestration definitions, prompts, helper assets, manifests, and BLAKE3 identities as built-in UI-loadable bundles and verify preview remains effect-free. r[lattice.guarded_agent_templates.rounds] r[lattice.guarded_agent_templates.cairn] r[lattice.guarded_agent_templates.octet]
- [x] [parallel] Add bounded status/replay evidence, UI/headless loading smoke tests without model calls, and explicit correctness/provider/exactly-once/crash/release non-claims. r[lattice.guarded_agent_templates.evidence.scenario.status] r[lattice.guarded_agent_templates.evidence.scenario.non-claim]

## Phase 5: Completion

- [x] [serial] Update README/example documentation with both reference codebases, bundle workflows, operator steps, authority grants, stop conditions, restoration recovery, and non-claims. r[lattice.guarded_agent_templates.evidence]
- [x] [serial] Run focused tests, `scripts/check.rs`, `octet-precommit-check`, Cairn validate and proposal/design/tasks gates, and bounded `nix flake check -L --option secret-key-files ''`; review any Octet identity transition with a fail-closed semantic migration before sync/archive. r[lattice.guarded_agent_templates.rounds] r[lattice.guarded_agent_templates.evidence]
