## Phase 1: Pure orchestration model

- [x] [serial] Add immutable orchestration definition, workflow revision, instance, lifecycle, fact, decision, proposal, approval, and budget types with BLAKE3 identities and serialization boundaries. r[lattice.agent_orchestration.model]
- [x] [serial] Implement the pure lifecycle/objective/termination decision core with explicit inputs and terminal-state rejection. r[lattice.agent_orchestration.core]
- [x] [parallel] Add positive and negative model tests for valid progression, invalid definitions, stale facts, duplicate facts, missing authority, and terminal mutation. r[lattice.agent_orchestration.model] r[lattice.agent_orchestration.core]

## Phase 2: Rounds, authority, and budgets

- [ ] [serial] Adapt orchestration rounds to immutable workflow revisions and the existing RunnerShell/Seaglass path without duplicating graph admission. r[lattice.agent_orchestration.rounds]
- [ ] [parallel] Add inert agent proposal artifacts plus approval admission bound to instance epoch, base revision, proposal, policy, and actor authority identities. r[lattice.agent_orchestration.authority]
- [ ] [parallel] Add pure multi-dimensional budget reservation and settlement, including missing-bound, insufficient-balance, duplicate-reservation, and observed-overage failures. r[lattice.agent_orchestration.budgets]

## Phase 3: Durable shell and integrations

- [ ] [serial] Add orchestration repository, event ledger, atomic reservation/outbox persistence, stable dispatch tokens, and restart reconciliation through narrow store and runner ports. r[lattice.agent_orchestration.durability]
- [ ] [parallel] Add pause, resume, cancellation, approval, rejection, and status command handling over the same pure decisions and persisted state. r[lattice.agent_orchestration.controls]
- [ ] [parallel] Bind schedule and webhook events to typed start/signal plans with trigger idempotency and overlap admission. r[lattice.agent_orchestration.triggers]

## Phase 4: Authoring, operations, and evidence

- [ ] [serial] Add typed Nickel orchestration contracts/import/export and deterministic rejection of unsafe references, malformed policies, plaintext secrets, and unsupported n8n orchestration export. r[lattice.agent_orchestration.format]
- [ ] [parallel] Add headless `orchestration validate|start|status|pause|resume|cancel|approve|reject|replay` commands with deterministic text/JSON output and stable exit behavior. r[lattice.agent_orchestration.cli]
- [ ] [parallel] Add chained redaction-safe orchestration events, side-effect-free replay, terminal summaries, and explicit exactly-once/provider-truth/Seaglass claim boundaries. r[lattice.agent_orchestration.events]

## Phase 5: Compatibility and verification

- [ ] [serial] Add a runnable positive multi-round agent example and negative fixtures for hidden cycles, stale approvals, authority escalation, revision drift, budget exhaustion, duplicate triggers, crash recovery, and replay tampering. r[lattice.agent_orchestration.rounds] r[lattice.agent_orchestration.authority] r[lattice.agent_orchestration.budgets] r[lattice.agent_orchestration.durability] r[lattice.agent_orchestration.events]
- [ ] [serial] Verify existing workflows, n8n parity fixtures, schedules, webhooks, replay, and the LLM operating graph remain compatible; run `scripts/check.rs`, Octet, Cairn gates/validation, and bounded Nix checks before sync/archive. r[lattice.agent_orchestration.format] r[lattice.agent_orchestration.triggers] r[lattice.agent_orchestration.cli]
