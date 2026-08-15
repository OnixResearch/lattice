# Operation-contract baseline, inventory, portfolio, and reference evidence

Date: 2026-08-15

## Goal and completion contract

The goal is one Lattice-owned production path from typed Nickel operation data to strict Rust admission, one canonical invocation plan, direct execution, workflow execution, typed output validation, bounded evidence, and preview-first Nickel promotion.

Observable completion requires:

- strict typed operation catalogs and data-only instances;
- domain-separated BLAKE3 identities for every authority-bearing layer;
- pure deterministic invocation and promotion planning;
- one shared invocation-plan shape for direct and workflow execution;
- execution through existing credential, runtime, Bounded Exec, run-identity, persistence, cancellation, and unknown-outcome boundaries;
- a read-only, explicit, bounded Cairn adapter;
- positive and negative tests for every changed boundary;
- passing focused, repository, Octet, Cairn, Nickel, Clippy, and Nix checks;
- accepted-spec sync and archive completion.

False completion includes a renamed shell node, schema-only validation, post-effect admission, separate direct and workflow planners, instance-supplied commands, executor loading during discovery, credential resolution during planning, untyped executor output, write-by-default promotion, or process success presented as external truth.

## Search and audit budget

The search used three serial, correlated lenses because the user did not authorize subagents:

1. Lattice-owned domain and runtime boundaries;
2. compatible published Onix components and existing Lattice adapters;
3. the reviewed Swamp type, definition, method, and invocation split.

The source budget was the Lattice source and lifecycle package, the Onix stack catalog, pinned component contracts already consumed by Lattice, and one immutable Swamp checkout. The tool budget was bounded native file search, focused reads, one external Git checkout, and deterministic repository checks. The implementation round budget is one preferred design, one adversarial audit, and one correction round before broad validation. Valid terminal states are validated, blocked, exhausted, or user-decision-required.

## Portfolio registry

### Family: Lattice-owned operation domain

- **Mechanism:** Add a narrow operation domain with pure catalog, instance, invocation, result, and promotion logic. Keep Nickel, files, credentials, processes, persistence, and publication in the shell.
- **Claim:** This preserves the required functional-core and imperative-shell boundary while reusing Lattice authority and runtime mechanisms.
- **Artifact:** New `src/operation/` components, typed Nickel contracts, an operation workflow node, direct CLI, and Cairn process adapter.
- **Evidence:** Existing `NodeConfig`, runner, executor, credential, process, headless, store, and publication boundaries are reusable.
- **Gap strength:** simpler.
- **Blocker:** none found.
- **Next check:** compile a minimal strict catalog and planner with positive and negative tests.
- **State:** active.

### Family: Instance-expanded shell behavior

- **Mechanism:** Treat each instance as a shell command plus metadata and lower it to `NodeConfig::Shell`.
- **Claim:** This would minimize new code.
- **Artifact:** No patch retained.
- **Evidence:** The design and specification explicitly reject instance behavior overrides and renamed shell nodes.
- **Gap strength:** stronger than the allowed instance authority.
- **Blocker:** It lets instance data define executable behavior and cannot prove direct/workflow plan parity.
- **Next check:** none.
- **State:** falsified.

### Family: Agent-evaluation command driver reuse

- **Mechanism:** Reuse the agent-evaluation JSON driver as the operation executor.
- **Claim:** Its absolute-path, clear-environment, bounded-output protocol could supply safe process mechanics.
- **Artifact:** `src/agent_evaluation/runtime/host.rs` and `src/process/shell.rs`.
- **Evidence:** The process mechanics fit, but the driver protocol owns evaluation trial identity, not operation type, instance, capability, output, or promotion meaning.
- **Gap strength:** equivalent only after replacing its domain contract.
- **Blocker:** Reusing the evaluation protocol would merge unrelated owners and still require a new operation contract.
- **Next check:** reuse only `run_bounded_clear_environment` and the existing `ProcessRunner` boundary.
- **State:** blocked as a whole; compatible mechanics selected.

### Family: Published Onix components

- **Mechanism:** Compose the already pinned Nickel evaluator, BLAKE3 identity mechanisms, Cap Root, `cap-std`, and Bounded Exec.
- **Claim:** Existing components own the required evaluation, lexical path, capability-directory, and process mechanics.
- **Artifact:** Existing Cargo and Nix pins in Lattice.
- **Evidence:** Lattice already consumes immutable revisions for `nickel-eval`, `content-identity-core`, `cap-root`, and `bounded-exec`.
- **Gap strength:** simpler.
- **Blocker:** None of these components owns Lattice operation policy, runtime authority, or result meaning.
- **Next check:** keep operation semantics in Lattice and reuse only exact component contracts.
- **State:** validated for composition.

## Adversarial selection

The selected route is the Lattice-owned operation domain composed with existing pinned mechanisms. The audit rejects these authority leaks:

- behavior, commands, capability widening, or non-claim removal in instance data;
- executor loading or credential resolution during discovery;
- runtime fallback, ambient executable or repository discovery, and unrestricted arguments;
- direct execution that does not reserve and persist a Lattice run identity;
- workflow lowering that reconstructs a different plan;
- output acceptance before contract checks;
- destination replacement, traversal, stale identities, or plaintext secrets during promotion;
- Cairn mutation methods or lifecycle-completion claims based only on exit status.

## Pre-change lifecycle evidence

Lattice policy pins Cairn revision `b6759012db4dab0158936e995b7bad39ac10465b`. Validation passed with no issues.

Initial gates passed:

- proposal receipt: `1b5787b0f59ac5cf0b64c9f761c98d29fcbc771b69354c4e555878fec7f16008`;
- design receipt: `012852502415b1eab804555edd7e21f926dc19093a9a4eb02986afa55ab7f278`;
- tasks receipt: `2c41a76ff0b91912ac7eb0ffbd79352b19a6cd10a2fc0b6d2a029f809ca9e072`.

## Pre-change test baseline

This focused command passed before core changes:

```text
CARGO_TARGET_DIR=target/operation-contracts-baseline cargo test -p lattice \
  --test test_workflow --test test_runner_core --test test_credentials \
  --test test_nickel_eval_adoption --test test_headless --test test_file_nodes \
  --test test_local_process --test test_doctor --test test_workflow_format \
  --test test_executor
```

Passed counts:

- workflow: 23;
- runner core: 21;
- credentials: 18;
- Nickel evaluator adoption: 4;
- headless: 8;
- file nodes and publication mechanics: 13;
- bounded local process: 21;
- doctor: 5;
- workflow format: 24;
- executor: 30.

Total: 167 tests passed, zero failed.

## Existing boundary inventory

- **Node kinds and schemas:** `src/engine/workflow.rs` owns `NodeConfig`; `src/node_kind_provider.rs` owns safe metadata and validation; `src/workflow_format.rs` owns strict portable and Nickel conversion.
- **Runtime binding:** `src/engine/runner.rs` owns `RuntimeBinding`, `RunnerPolicy`, runtime admission, typed ports, and output-shape checks.
- **Execution:** `src/engine/executor.rs` owns node execution, effect binding, cache identities, output mapping, cancellation, and redaction.
- **Credentials:** `src/credentials.rs` owns opaque `SecretRef`, `NodeCredentialRefs`, provider validation, execution-only resolution, and redaction.
- **Processes:** `src/process/core.rs` owns pure process admission; `src/process/shell.rs` uses pinned Bounded Exec, positive limits, cancellation, truncation rejection, and process-group teardown.
- **Run identity and persistence:** `src/store/run/records.rs` atomically reserves global run IDs; `src/headless/shell.rs` records bounded run, detail, cache, and replay facts.
- **Nickel:** `src/nickel/mod.rs`, `src/config/mod.rs`, and `src/workflow_format.rs` keep evaluation and file access outside pure conversion logic.
- **Publication:** `src/workflow_template/init.rs` and `src/engine/effect/runtime.rs` show create-only and capability-relative write mechanics.
- **Doctor:** `src/main.rs` gathers shell facts; `src/doctor/core.rs` evaluates safe diagnostics.
- **Agent metadata:** existing host-provided adapter boundaries can expose metadata without executor authority.

## Swamp reference boundary

Reviewed source:

- repository: `https://github.com/swamp-club/swamp`;
- revision: `a648c1307fd5cffa029cb8348b2ec20cb727bd66`;
- license: AGPL-3.0-only with the repository exception file;
- reviewed surfaces: `README.md`, `src/domain/models/model.ts`, `src/domain/definitions/definition.ts`, and `src/domain/models/model_invocation_service.ts`.

Adopted ideas:

- reusable type definitions separate from configured instances;
- exact type versions, named methods, argument routing, and output specifications;
- method discovery separate from invocation;
- explicit preflight and bounded invocation structure.

Rejected behavior:

- dynamic TypeScript loading and lazy executor imports;
- YAML and CEL as canonical authoring;
- random UUID authority identities;
- ambient current-directory and environment discovery;
- instance mutation, automatic definition creation, and runtime tag overrides;
- raw logs, stack traces, and provider data as ordinary public output;
- extension-defined execution authority and direct runtime dependencies.

This work is clean-room. It copies no Swamp source and adds no Swamp runtime or build dependency. Public-license compatibility does not establish semantic compatibility, authority, or evidence strength.

## Claim boundary

The baseline proves only that the listed pre-change Lattice surfaces pass their current tests. The reference review proves only the inspected source identity and bounded design observations. It does not prove Swamp correctness, Lattice correctness, external-system truth, executor correctness, sandboxing, effect uniqueness, workflow correctness, production readiness, or release eligibility.
