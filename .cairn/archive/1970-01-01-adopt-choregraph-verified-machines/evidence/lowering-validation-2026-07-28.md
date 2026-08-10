# Choregraph lowering validation — 2026-07-28

## Exact source

```text
rid=rad:zL2ncTUeASVYwcoGkEXv9JKgGbAF
revision=81b260da2256147ec82cb5dd44c0a0c1dcb7500d
candidate=c493ef88a648eb30b621e74fa5c0f7294df9048501a379395403ad3dcd8a4c65
source-archive-blake3=41adfbdae4bc5b688ef92ab11cd39ed9c8f572bf6214f522d9b4a1290e5f84d5
release-bundle-blake3=6912c485313908411040fa533bd75a1cfd2611d7589605268ae111b3ba1472bc
```

Cargo, Crane, Nix, and the governed source-admission fixture select the same revision. Generated Cargo and Nix lockfiles bind that revision. No floating, GitHub, sibling, vendor, runtime, or subprocess fallback exists.

## Lowering boundary

`src/workflow_format/choregraph.rs` is the pure conversion core. It converts a decoded `PortableWorkflow` into the closed `lattice-workflow-v1` input and calls `choregraph_nickel::lower_evaluated_json` with reviewed limits.

`load_workflow_nickel_file` remains the shell. It owns Nickel evaluation and file-backed Markdown prompt loading. The shell requires Choregraph lowering after decoded identity validation and before product conversion completes.

The source-identity Nix check restricts `choregraph_nickel` use to the workflow-format boundary. It rejects file, process, network, environment, asynchronous, and unsafe markers in the pure module.

## Positive and negative checks

The focused workflow-format suite passed 23 tests. The added positive test requires exact node, edge, source-format, and empty import-closure facts. The added negative test adds a validly shaped reverse edge and requires Choregraph cycle rejection.

The runner suite passed 21 tests. The persisted migration suite passed 3 tests. Focused library webhook checks and scoped Clippy passed.

`scripts/check.rs` passed its operator E2E, formatting, complete tests, scoped Clippy, and accepted-spec checks.

The exact Choregraph source check passed. Its floating, GitHub, and skipped-proof fixture failed as required.

## Octet

The new pure lowering module introduced no Octet finding. Seven earlier unaccepted findings in the machine-cutover and webhook conversion were removed without suppression or policy changes.

The reviewed migration compared the prior and current semantic finding sets:

```text
old=1141 candidate=1141 matched=1141 unmatched_old=0 unmatched_candidate=0
```

The regenerated baseline has identity `b3:a590a1553486861cec652bdc1df4831b5afece511f8112f3cfd2122678a32088`. The maintained wrapper reports 892 accepted current identities, zero new identities, and zero stale identities.

The raw child records `integration-failure` because deny-level accepted findings make Cargo exit after emitting 1,141 findings. The maintained wrapper passes by exact baseline policy. This is full lint coverage with accepted debt, not a clean raw Octet result.

## Broad Nix gate

`nix flake check -L --option secret-key-files ''` passed all checks on `x86_64-linux`. Nix reported the other default flake systems as incompatible and omitted them. This result does not claim cross-system validation.

## Lifecycle dependency correction

The original metadata named two independent external-source roadmap changes as hard dependencies. Neither change is implemented. Choregraph supersedes the planned external graph cutover and deliberately retains Lattice's evaluator shell.

The stale dependencies were removed instead of marking unimplemented tasks complete or archiving false evidence. Both roadmap changes remain active and independent.

## Cairn sync and traceability

Cairn synced all 13 Choregraph adoption requirements and their 13 scenarios into `cairn/specs/choregraph-adoption/spec.md` after the dependency correction. The accepted-spec shape test also passed after the reviewed sync added an ID to each scenario.

Tracey then reported 952 referenced requirements out of 964, with no dangling identifiers. All 26 Choregraph adoption requirement and scenario IDs are referenced. The 12 remaining gaps belong to the independent Bounded Exec Radicle-source change.

## Claim boundary

These checks prove exact source selection, the selected pure conversion path, tested rejection, and current maintained gate results. They do not prove arbitrary Nickel equivalence, evaluator soundness, runtime effects, persistence correctness, authorization, external services, whole-Lattice correctness, or release eligibility.
