# Add Steel contract backend planning

## Why

`ucan-nickel-contracts` currently focuses on Nickel-authored policy and generated contract artifacts. That is the right static policy/config seam, but UCAN integrations also need a way to talk to Steel Scheme the same way they can talk to Nickel: through a reviewed, deterministic contract boundary instead of embedding ad hoc runtime calls in downstream applications.

Steel support should not turn this crate into an orchestration engine or grant Scheme ambient authority. The crate needs a language-neutral contract envelope, a Steel backend descriptor/evaluation request shape, and higher-order callable contract descriptors that let UCAN reason about Scheme predicates/functions while Rust remains the enforcement and receipt authority.

## What changes

- Define Steel as a sibling contract backend to Nickel under a shared UCAN contract envelope.
- Add DTO and policy requirements for backend identity, normalized source hashes, evaluation request/response receipts, and deny-by-default unsupported backend behavior.
- Add higher-order contract descriptors for callable/predicate boundaries, including arity, argument/return schemas, required UCAN capabilities, determinism/effect class, and content hash metadata.
- Specify that Steel receives only bounded DTO inputs and returns structured allow/deny/evidence; Rust validates authority and receipts and performs no ambient host effects for Steel.
- Document that Nickel remains the static policy/artifact backend while Steel is the dynamic predicate/callable backend.

## Scope

In scope: native Cairn requirements/design/tasks for the Steel contract backend, shared contract envelope, higher-order descriptors, authority/receipt constraints, and future Rust/Nickel/fixture verification rails.

Out of scope: implementing the Steel interpreter adapter in this planning slice, adding ambient host functions, replacing Nickel policy export, adding remote/network execution, provisioning credentials, or claiming sandbox isolation beyond the explicit Rust-owned DTO/authority boundary.

## Validation

Validate this package with `cairn validate --root .` and proposal/design/tasks gates. Implementation follow-up should add positive and negative Rust fixtures for Nickel and Steel backend descriptors, generated policy freshness checks, and receipt hash stability checks.
