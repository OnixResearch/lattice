## Why

Lattice has typed built-in nodes, generic runtime bindings, credential references, and admitted workflow execution. Its current operation surface remains a closed product enum. An agent or operator cannot inspect one reusable external-system contract, instantiate it as reviewed data, invoke one named method, and promote that exact invocation into a Nickel workflow.

Swamp demonstrates a useful product split between reusable typed behavior, data-only instances, and named methods. Lattice will adapt that split through Nickel, Rust, BLAKE3, explicit capability policy, and existing bounded runtime shells. It will not copy Swamp source, load TypeScript dynamically, adopt YAML or CEL, or grant authority from schema metadata.

## What Changes

- Add a typed Nickel and Rust operation-type catalog with versioned methods, argument and output contracts, preflight declarations, capability classes, runtime bindings, and non-claims. r[lattice.operations.catalog]
- Separate reusable operation types from data-only operation instances with exact type revisions, target facts, tags, and opaque credential references. r[lattice.operations.instances]
- Add pure invocation admission and preflight planning before any process, network, filesystem, credential, clock, or persistence effect. r[lattice.operations.preflight]
- Add direct one-off operation execution through existing Lattice runtime, credential, budget, event, persistence, and unknown-outcome boundaries. r[lattice.operations.direct_run]
- Add preview-first promotion of one admitted invocation into a reviewed Nickel workflow with exact operation and input identities. r[lattice.operations.promotion]
- Dogfood the contract with a read-only Cairn adapter before adding mutable infrastructure methods. r[lattice.operations.pilot]

## Success Criteria

- An operator can inspect one operation type without loading its executor or resolving credentials.
- An instance cannot change the declared methods, schemas, capabilities, runtime, or non-claims of its type.
- Invalid arguments, missing runtime bindings, capability mismatches, stale type revisions, and malformed credential references fail before effects.
- Direct execution produces the same typed invocation plan used by a promoted workflow node.
- Promotion previews deterministic Nickel and writes only after explicit execution.
- The initial Cairn adapter supports read-only lifecycle inspection with bounded output and no mutation command.
- Positive and negative tests cover catalog, instances, admission, direct execution, promotion, redaction, and adapter failures.

## False Completion

A renamed shell node is not a typed operation contract. A schema without an exact executor identity, capability declaration, and preflight plan is not completion.

A generated workflow that loses type revision, arguments, credential references, runtime policy, or output contracts is not completion. A successful command exit alone does not establish semantic correctness or external truth.

## Impact

- **Lattice core**: pure operation DTO validation, catalog reduction, instance binding, invocation planning, promotion planning, and deterministic diagnostics.
- **Lattice shell**: Nickel loading, runtime registry binding, credential resolution at dispatch, direct run, preview-first file publication, and bounded Cairn process adapter.
- **Choregraph**: receives lowered graph nodes and typed ports only. It does not load operation catalogs or execute methods.
- **Animus**: can consume metadata-only operation descriptions through existing host-provided tool contracts. It does not gain executor or workflow authority.
- **Testing**: baseline and post-change operation, workflow, credential, runner, bounded-exec, Nickel, Cairn, Octet, and Nix rails.

## Out of Scope

- A public operation registry, release channels, operation-pack installation, remote discovery, or automatic updates.
- A queryable cross-run data plane, generic post-run analyzers, or Valence evidence projection.
- Mutable OnixOS deployment operations, generic manual approval nodes, or secret storage.
- Claims for external-system correctness, executor correctness, sandboxing, exactly-once effects, production readiness, or release eligibility.

## Affected Specs

- `operation-contracts`: catalog, instances, preflight, direct execution, promotion, and read-only Cairn pilot.
