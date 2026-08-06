## Why

Lattice's UI template picker currently creates only individual shell nodes. Whole workflow and orchestration examples exist under `examples/`, but operators must discover paths and import or start them from the CLI. Treating a multi-round agent loop as one shell-node template would hide iteration, authority, budget, approval, and evidence boundaries inside an opaque child process.

Lattice needs a distinct UI-loadable workflow-template bundle model that reuses the existing workflow and orchestration validators, keeps selection side-effect free, and requires explicit admission before persistence or execution.

## What Changes

- Add a typed Nickel workflow-template bundle format, separate from the existing shell `NodeTemplate` format.
- Bind each bundle manifest, workflow, optional orchestration definition, and support asset to explicit BLAKE3 content identities.
- Add a pure bundle-validation and application-planning core over parsed in-memory inputs and explicit store-conflict facts.
- Add a thin filesystem shell that loads config-adjacent bundle directories without path escape, symlink escape, hidden network retrieval, or partial catalog poisoning.
- Add a TUI workflow-template browser that previews a validated bundle as an unsaved draft and creates no workflow, orchestration instance, agent call, or other effect on selection or cancellation.
- Add an explicit confirmation path that reuses existing workflow save/import and orchestration start admission, rejects name/revision drift before mutation, and never starts an orchestration merely because its template was selected.
- Extend fresh initialization with create-only built-in workflow-template bundle files while preserving existing config and template bytes on collision.
- Preserve the existing `Ctrl-N` shell-node template catalog and CLI workflow/orchestration behavior.

## Success Criteria

- A fresh initialized configuration exposes every valid built-in workflow bundle in the TUI without diagnostics.
- Selecting a workflow bundle loads its exact workflow into a previewable unsaved draft and performs no persistent or external effect.
- An optional orchestration draft can start only after explicit operator confirmation and exact workflow-revision, policy, authority, runtime, credential, effect, trigger, and budget admission.
- Malformed Nickel, undeclared assets, BLAKE3 drift, path escape, workflow-name collision, orchestration revision drift, missing authority, and cancellation fail without partial workflow or orchestration mutation.
- Positive and negative tests cover pure validation, loading, initialization, TUI preview/cancel, activation admission, idempotent exact matches, and incompatible conflicts.
- Existing node templates, stored workflows, headless imports/runs, and orchestration commands remain compatible.

## Non-Goals

- Downloading templates, a remote marketplace, plugin installation, or network-backed template discovery.
- Executing arbitrary Steel or shell agent loops as opaque template-loader behavior.
- Automatic orchestration start, automatic approval, automatic credential selection, or agent-authored authority expansion.
- Hidden cyclic workflow execution, n8n orchestration parity, remote exactly-once effects, or production-readiness claims.
- Replacing the existing workflow, runtime, Seaglass, orchestration, trigger, credential, budget, event, or persistence validators.

## Impact

- **Files**: workflow-template model/core/shell modules, Nickel contracts and built-in bundle assets, initialization, TUI state/rendering/input handling, focused tests, examples, documentation, and Cairn traceability artifacts.
- **Compatibility**: additive workflow-template catalog; existing shell node templates and CLI surfaces retain their current semantics.
- **Security/authority**: template discovery and preview are read-only; persistence and execution remain explicit admitted operations through existing Lattice boundaries.
- **Testing**: establish the relevant baseline before core edits; add positive and negative unit/integration/TUI tests; run `scripts/check.rs`, Octet, Cairn gates/validation, and bounded Nix checks.
