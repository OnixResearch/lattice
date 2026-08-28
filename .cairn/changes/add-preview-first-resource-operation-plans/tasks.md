# Tasks: Add preview-first resource operation plans

## Pure contract

- [ ] [serial] Add nominal resource, content, replica, locator, observation, plan, attempt, and outcome roles to `lattice-core`. r[lattice.resource_operations.roles]
- [ ] [serial] Add domain-separated full-width BLAKE3 plan identity framing. r[lattice.resource_operations.plan]
- [ ] [serial] Reject raw paths, sampled fingerprints, missing identities, stale observations, conflicting roles, and unsafe bounds. r[lattice.resource_operations.plan]
- [ ] [serial] Add closed copy, move, materialize, derivative, and replica-delete effect plans. r[lattice.resource_operations.effects]

## Shell composition

- [ ] [serial] Add an application-owned catalog-facts port and a finite fake implementation. r[lattice.resource_operations.ports]
- [ ] [serial] Compose exact plan persistence, durable approval, currentness recheck, and effect dispatch. r[lattice.resource_operations.approval]
- [ ] [serial] Keep catalog storage, filesystem, cloud, transport, clock, credential, and UI types outside the pure core. r[lattice.resource_operations.ports]
- [ ] [serial] Record completed, denied, failed, cancelled, and unknown outcomes through existing durable attempt semantics. r[lattice.resource_operations.outcomes]

## Verification and documentation

- [ ] [parallel] Add positive pure tests for each admitted operation class. r[lattice.resource_operations.verification]
- [ ] [parallel] Add negative pure tests for role substitution, stale facts, sampled fingerprints, conflicts, limits, and missing approval. r[lattice.resource_operations.verification]
- [ ] [parallel] Add shell tests that prove no protected effect occurs before approval and current-state recheck. r[lattice.resource_operations.verification]
- [ ] [parallel] Add cancellation, uncertain-outcome, and explicit duplicate-risk retry tests. r[lattice.resource_operations.verification]
- [ ] [serial] Document the resource-catalog owner, Lattice boundary, recovery states, and non-claims. r[lattice.resource_operations.evidence]
- [ ] [serial] Run focused Cargo, Clippy, Octet, Cairn validation, and proposal, design, and tasks gates. r[lattice.resource_operations.evidence]
