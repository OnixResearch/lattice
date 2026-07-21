## Phase 1: Lifecycle and core

- [x] [serial] Record the live interrupted/successful dogfood evidence, exact claim boundary, alternatives, and the explicit abandonment contract. r[lattice.agent_orchestration.unobserved_abandon]
- [x] [serial] Add the pure abandonment action, event, authority/cancellation/identity validation, acknowledgment-sensitive budget settlement, and fixed terminal failure state. r[lattice.agent_orchestration.unobserved_abandon]
- [x] [serial] Add positive and negative core tests for acknowledged and planned abandonment, missing cancellation, mismatched dispatch, unauthorized actor, duplicate/terminal facts, and deterministic replay. r[lattice.agent_orchestration.unobserved_abandon]

## Phase 2: Shell and operator controls

- [x] [serial] Add the shell boundary that refuses abandonment when a matching persisted run exists and commits only the unobserved core fact. r[lattice.agent_orchestration.unobserved_abandon]
- [x] [serial] Add `orchestration abandon` with actor and text/JSON output while preserving existing recovery and cancellation commands. r[lattice.agent_orchestration.unobserved_abandon]
- [x] [serial] Add shell and CLI tests for the two-phase recover/cancel/abandon flow, observed-run rejection, unauthorized use, repeat invocation, and safe output. r[lattice.agent_orchestration.unobserved_abandon]
- [x] [serial] Document the operator sequence, conservative budget semantics, and explicit remote-effect non-claims. r[lattice.agent_orchestration.unobserved_abandon]

## Phase 3: Validation and closure

- [ ] [serial] Run focused tests, `scripts/check.rs`, Octet, Cairn gates/validation, and bounded Nix validation; sync the accepted spec and archive with current evidence. r[lattice.agent_orchestration.unobserved_abandon]
