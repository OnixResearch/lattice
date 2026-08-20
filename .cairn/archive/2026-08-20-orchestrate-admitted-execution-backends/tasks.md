## Phase 1: Product contract and planner mapping

- [x] [depends:introduce-typed-operation-contracts] Add execution requirement and backend plan identities to Lattice operation contracts and Nickel workflow schemas. r[lattice.execution_backend_orchestration.contract]
- [x] [serial] Pin a published Animus planner revision and freeze positive and negative compatibility fixtures before direct dependency selection. r[lattice.execution_backend_orchestration.planner_mapping]
- [x] [depends:lattice.execution_backend_orchestration.contract] Implement pure plan revalidation against Lattice runtime, adapter, authority, credential-reference, budget, persistence, and effect facts. r[lattice.execution_backend_orchestration.admission]
- [x] [depends:lattice.execution_backend_orchestration.admission] Persist the exact selected route and attempt identity before effects. r[lattice.execution_backend_orchestration.persistence]

## Phase 2: Dispatch, recovery, and operator evidence

- [x] [depends:lattice.execution_backend_orchestration.persistence] Bind dispatch, retry, resume, and recovery to the persisted backend plan and deny silent substitution. r[lattice.execution_backend_orchestration.dispatch]
- [x] [parallel] Add positive fixtures for each supported backend class and existing bounded runtime adapter. r[lattice.execution_backend_orchestration.verification]
- [x] [parallel] Add negative fixtures for stale plans, missing adapters, authority or credential denial, budget mismatch, backend drift, retry substitution, and unknown outcomes. r[lattice.execution_backend_orchestration.verification]
- [x] [serial] Add safe CLI/TUI explanation, run focused checks and crash-boundary tests, validate Cairn gates, and document non-claims. r[lattice.execution_backend_orchestration.inspection] r[lattice.execution_backend_orchestration.verification]
