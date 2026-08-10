## Phase 1: Baseline and contract inventory

- [ ] [serial] O1 Run and record current workflow, runner, credential, Nickel, headless, promotion-like publication, bounded-exec, doctor, and Cairn lifecycle baselines before core changes. r[lattice.operations.catalog] r[lattice.operations.direct_run]
- [ ] [serial] O2 Inventory every current node-kind, runtime-binding, schema, credential, port, output-validation, run-record, and workflow-publication path that the operation contract will reuse. r[lattice.operations.catalog] r[lattice.operations.preflight]
- [ ] [parallel] O3 Record the exact Swamp reference surfaces, AGPL boundary, clean-room adopt/reject decisions, and no-runtime-dependency rule. r[lattice.operations.catalog]

## Phase 2: Typed catalog and instances

- [ ] [serial] C1 Add typed Nickel contracts for operation types, methods, arguments, outputs, preflight declarations, capabilities, runtime bindings, credential bindings, bounds, and non-claims. r[lattice.operations.catalog]
- [ ] [serial] C2 Add strict Rust DTO validation with unknown-field, duplicate, limit, schema-version, capability, runtime, and non-claim diagnostics. r[lattice.operations.catalog] r[lattice.operations.preflight]
- [ ] [serial] C3 Add domain-separated BLAKE3 identities for catalogs, types, type revisions, instances, methods, schemas, invocation plans, and promotion plans. r[lattice.operations.catalog] r[lattice.operations.instances]
- [ ] [serial] C4 Add data-only operation instances that bind exact type revisions, target facts, tags, defaults, and opaque credential references without behavior or capability overrides. r[lattice.operations.instances]
- [ ] [parallel] C5 Add positive and negative catalog and instance fixtures for valid data, duplicates, drift, unknown fields, unsafe overrides, malformed references, and named-limit boundaries. r[lattice.operations.catalog] r[lattice.operations.instances]

## Phase 3: Pure admission and shared execution plan

- [ ] [serial] P1 Add a pure invocation planner over validated type, instance, method, arguments, runtime facts, capability policy, credential-reference facts, and bounds. r[lattice.operations.preflight]
- [ ] [serial] P2 Return one ordered diagnostic set and no invocation plan for missing methods, schema errors, capability widening, stale identities, unavailable runtimes, invalid credential use, or bound failures. r[lattice.operations.preflight]
- [ ] [serial] P3 Add the operation workflow node or equivalent lowering that uses the same canonical invocation plan and typed ports as direct execution. r[lattice.operations.direct_run]
- [ ] [parallel] P4 Add parity tests that fail when direct and workflow paths differ in plan identity, runtime binding, credential bindings, output contracts, diagnostics, or non-claims. r[lattice.operations.direct_run]

## Phase 4: Direct execution and Cairn pilot

- [ ] [serial] D1 Add headless operation type, instance, method, and schema inspection without executor loading or credential resolution. r[lattice.operations.catalog]
- [ ] [serial] D2 Add `operation run` through existing run reservation, runtime registry, credential dispatch, budgets, cancellation, persistence, redaction, and unknown-outcome boundaries. r[lattice.operations.direct_run]
- [ ] [serial] D3 Validate executor results against the declared output contract before downstream use or success reporting. r[lattice.operations.direct_run]
- [ ] [serial] D4 Add a read-only Cairn adapter for bounded repository status and lifecycle validation through the injected Bounded Exec path. r[lattice.operations.pilot]
- [ ] [parallel] D5 Add negative Cairn cases for mutation method requests, arbitrary arguments, missing executable, unsafe root, timeout, cancellation, truncation, malformed JSON, nonzero exit, and overclaiming. r[lattice.operations.pilot]

## Phase 5: One-off promotion to Nickel

- [ ] [serial] N1 Add a pure promotion planner that consumes one exact admitted invocation and emits deterministic Nickel content, destination facts, content BLAKE3, and blockers. r[lattice.operations.promotion]
- [ ] [serial] N2 Add preview-first CLI output and explicit capability-relative publication that refuses existing destinations and stale invocation or contract identities. r[lattice.operations.promotion]
- [ ] [serial] N3 Preserve type revision, instance data, method, arguments, runtime policy, output contracts, and credential references while excluding resolved secrets and ambient values. r[lattice.operations.promotion]
- [ ] [parallel] N4 Add positive direct-run-to-workflow parity and negative stale, unrepresentable, collision, traversal, existing-file, secret, and contract-drift fixtures. r[lattice.operations.promotion]

## Phase 6: Product integration and closeout

- [ ] [parallel] I1 Add safe operation metadata projection through existing Animus-compatible host tool descriptors without granting an executor or operation authority. r[lattice.operations.catalog] r[lattice.operations.preflight]
- [ ] [serial] I2 Extend repository validation and doctor facts for catalog freshness, runtime bindings, credential-reference shape, and Cairn pilot readiness without loading secrets. r[lattice.operations.catalog] r[lattice.operations.pilot]
- [ ] [serial] I3 Document operation authoring, inspection, direct execution, promotion, capability boundaries, Cairn pilot use, and non-claims. r[lattice.operations.catalog] r[lattice.operations.promotion]
- [ ] [serial] I4 Run focused tests, formatting, Clippy, `scripts/check.rs`, Octet, Cairn validation and proposal/design/tasks gates, and bounded Nix validation. r[lattice.operations.catalog] r[lattice.operations.direct_run] r[lattice.operations.pilot]
- [ ] [serial] I5 Review dry-run sync and archive plans, make sure that every requirement reaches the accepted operation-contracts spec, and archive only after current positive and negative evidence passes. r[lattice.operations.promotion]
