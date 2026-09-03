# Tasks: Pilot linear process-permit flow

## Dependencies and baseline

- [ ] [serial] Confirm Choregraph change `admit-linear-resource-flow` is archived and record its immutable handoff revision and identities. [depends:lattice.linear_process_permit.dependencies]
- [ ] [serial] Record the linked Trellis proof profile and Kamacite artifact profile identities and non-claims. r[lattice.linear_process_permit.dependencies]
- [ ] [serial] Record local-process, effect-intent, Basalt, persistence, retry, replay, crash-window, Bounded Exec, Octet, and lifecycle baselines. r[lattice.linear_process_permit.evidence]
- [ ] [parallel] Audit existing pure state, attempt, storage, authorization, and process ports for reuse before adding any new abstraction. r[lattice.linear_process_permit.boundary]

## Pure core and intake

- [ ] [depends:lattice.linear_process_permit.dependencies] Add the opt-in local shell-node process-permit profile and exact producer identity contract. r[lattice.linear_process_permit.profile]
- [ ] [depends:lattice.linear_process_permit.profile] Implement pure Choregraph handoff admission without row unification or graph reinterpretation. r[lattice.linear_process_permit.intake]
- [ ] [depends:lattice.linear_process_permit.intake] Define `Available`, `Owned`, `Settled`, and `Unknown` states with exact permit, attempt, observation, and uncertainty identities. r[lattice.linear_process_permit.state]
- [ ] [depends:lattice.linear_process_permit.state] Implement pure preparation, settlement, uncertainty, abandonment, later-settlement, and fresh-retry decisions. r[lattice.linear_process_permit.transition]
- [ ] [depends:lattice.linear_process_permit.transition] Make `Settled` and `Unknown` terminal for the original permit occurrence. r[lattice.linear_process_permit.single_use]
- [ ] [parallel] Keep the core free of filesystem, process, storage, clock, network, credential, authorization-service, and presentation effects. r[lattice.linear_process_permit.boundary]

## Shell orchestration and persistence

- [ ] [depends:lattice.linear_process_permit.intake] Require exact Basalt authorization after handoff admission and before durable process preparation. r[lattice.linear_process_permit.authorization]
- [ ] [depends:lattice.linear_process_permit.transition] Persist exact `Owned` permit and attempt state before Bounded Exec starts a process. r[lattice.linear_process_permit.ordering]
- [ ] [depends:lattice.linear_process_permit.ordering] Reconcile uncertain storage commit outcomes before any process execution. r[lattice.linear_process_permit.ordering]
- [ ] [depends:lattice.linear_process_permit.ordering] Route the admitted process through the existing pinned Bounded Exec adapter. r[lattice.linear_process_permit.execution]
- [ ] [depends:lattice.linear_process_permit.execution] Normalize terminal observations and settle only exact owned attempts. r[lattice.linear_process_permit.settlement]
- [ ] [depends:lattice.linear_process_permit.execution] Persist `Unknown` for missing, stale, foreign, conflicting, or untrustworthy terminal evidence. r[lattice.linear_process_permit.uncertainty]
- [ ] [depends:lattice.linear_process_permit.uncertainty] Deny automatic retry, permit restoration, and occurrence reuse from `Unknown`. r[lattice.linear_process_permit.single_use]
- [ ] [depends:lattice.linear_process_permit.uncertainty] Add explicit settle, abandon, and new-permit retry actions with duplicate-risk acknowledgment. r[lattice.linear_process_permit.recovery]

## Tests and evidence

- [ ] [parallel] Add positive pure-core tests for admission, ownership, success settlement, failure settlement, uncertainty, later settlement, abandonment, fresh retry, and replay. r[lattice.linear_process_permit.evidence]
- [ ] [parallel] Add negative core tests for stale identities, mismatches, duplicate binding, terminal reuse, missing acknowledgment, and implicit legacy adoption. r[lattice.linear_process_permit.evidence]
- [ ] [parallel] Add shell tests for authorization denial, persistence-before-process order, commit uncertainty, Bounded Exec failure, and no protected effect after denial. r[lattice.linear_process_permit.evidence]
- [ ] [depends:lattice.linear_process_permit.execution] Add supported crash-window tests and deterministic metadata-only receipts. r[lattice.linear_process_permit.uncertainty] r[lattice.linear_process_permit.evidence]
- [ ] [depends:lattice.linear_process_permit.recovery] Add operator inspection and recovery tests that retain the historical unknown outcome. r[lattice.linear_process_permit.recovery]

## Compatibility and closeout

- [ ] [parallel] Prove legacy workflows, persistence, commands, scheduler capacity, and other effect classes remain unchanged without explicit opt-in. r[lattice.linear_process_permit.compatibility]
- [ ] [parallel] Update Choregraph effect authority, bounded local process, storage uncertainty, recovery, workflow boundaries, README, and non-claim documentation. r[lattice.linear_process_permit.evidence]
- [ ] [serial] Run focused process, executor, persistence, replay, crash, and Choregraph migration tests. Then run repository guards, Clippy, Octet, and Nix validation. r[lattice.linear_process_permit.evidence]
- [ ] [serial] Validate Cairn proposal, design, tasks, and repository state. Then sync accepted requirements and archive with fresh evidence. r[lattice.linear_process_permit.evidence]
