## Context

The existing `NodeTemplate` catalog is intentionally narrow: Nickel files describe one node, command interpolation produces one shell command, and `App::add_template_node` rejects every non-shell kind. Whole workflows are imported through existing format code, and durable orchestration definitions are validated and started through separate headless commands. The TUI has no orchestration surface and does not discover whole examples.

The new capability must not widen `NodeTemplate` into an ambiguous union. A workflow-template bundle has different identity, loading, preview, persistence, and authority semantics and therefore receives a separate model and UI state machine.

## Success Contract

Completion requires deterministic evidence that a valid local bundle can be discovered, validated, previewed, saved, and explicitly started while malformed or conflicting bundles cannot partially mutate workflow/orchestration state. Merely listing example names, invoking a shell script, or passing Nickel evaluation is insufficient.

False completion includes:

- presenting a shell node that internally owns multiple agent rounds;
- loading a workflow but silently dropping its orchestration policy or support assets;
- starting an orchestration on template selection;
- trusting a workflow name when its content differs from the bound revision;
- treating one malformed bundle as authority to hide or reject unrelated valid bundles;
- changing initialized user files when a target already exists.

## Architecture

```text
config-adjacent workflow-templates/<bundle-id>/
  manifest.ncl
  workflow.ncl
  orchestration.ncl          # optional
  assets/...                 # optional declared support files
        |
        v
WorkflowTemplateShell
  bounded directory/file reads + Nickel evaluation
        |
        v
WorkflowTemplateCore (pure)
  manifest + file bytes + explicit limits/conflict facts
  -> validated bundle | diagnostics | application plan
        |
        v
TUI catalog -> preview draft -> explicit save/start commands
        |
        v
existing workflow/orchestration admission and store shells
```

The core performs no filesystem, environment, process, network, database, clock, terminal, or UI operation. The shell captures bytes and metadata under named limits, then passes owned values to the core. The TUI consumes pure catalog/application results and delegates persistence/execution to existing command ports.

## Typed Bundle Model

A Nickel-authored manifest declares:

- schema version, stable bundle id, display name, description, and category;
- the relative workflow file and its BLAKE3 byte identity;
- zero or one relative orchestration file and its BLAKE3 byte identity;
- every support asset's safe relative path, media kind, byte identity, and required/optional status;
- explicit compatibility requirements for workflow schema, orchestration schema, node/runtime kinds, and minimum Lattice feature identities;
- redaction-safe operator notes and non-claims.

The manifest cannot contain resolved credentials, bearer tokens, private keys, actor proofs, executable bytes, network URLs for retrieval, absolute paths, parent traversal, or implicit glob imports. All numeric schema versions, count limits, and byte limits are named constants or configuration values.

A `ValidatedWorkflowTemplateBundle` contains parsed workflow and optional orchestration values plus canonical BLAKE3 identities for the manifest and declared bytes. An orchestration draft must bind the exact workflow revision supplied by the bundle. Bundle identity does not certify workflow behavior, agent correctness, policy correctness, or release eligibility.

## Pure Validation and Application Planning

The functional core is split into small deterministic operations:

1. Validate manifest shape, identifiers, declared paths, limits, and duplicate entries.
2. Match the complete supplied file set against declarations; reject missing required files, undeclared files, duplicate normalized paths, and content-identity drift.
3. Reuse workflow and orchestration parsers/validators over in-memory bytes.
4. Verify optional orchestration references bind the exact parsed workflow revision and compatible runtime/effect/authority/budget requirements.
5. Compare explicit store facts and produce `Preview`, `ExactAlreadyPresent`, `CreateDraft`, or a stable conflict diagnostic.

Application planning must be testable with plain values and no mocks. It never writes a file, saves a workflow, starts an instance, resolves a credential, or invokes a node runtime.

## Imperative Loading Shell

The shell enumerates immediate bundle directories in deterministic name order. It opens the configured template root as a capability, admits manifest-declared lexical relative paths, reads only regular files beneath that root, rejects symlinks and escape attempts, and enforces named per-file, per-bundle, and catalog count/byte limits.

One invalid bundle produces one bounded diagnostic and does not suppress unrelated valid bundles. Diagnostics include the safe bundle-relative path and stable error class, not host paths, file contents, credential material, or Nickel evaluation internals that may expose source values. No loader operation performs network retrieval or executes bundle content.

## TUI State and Mutation Boundary

The TUI receives a sorted validated catalog through the same application construction boundary used for node templates. A distinct workflow-template browser shows name, description, category, compatibility, whether an orchestration draft exists, and non-claims.

State transitions are explicit:

```text
Closed -> Browse -> Preview -> UnsavedWorkflowDraft
                    | cancel -> Browse/Closed
UnsavedWorkflowDraft -> explicit Save -> existing workflow save admission
Saved exact revision -> explicit Start -> existing orchestration start admission
```

Selection and preview only replace the current unsaved canvas after the normal dirty-workflow confirmation; they create no store record or external effect. Cancellation preserves the prior workflow and store bytes. The optional orchestration remains an inert draft until the exact workflow revision is saved and an operator explicitly requests start with an admitted actor authority.

Before start, Lattice revalidates the current saved workflow hash, orchestration hash, runtime bindings, effects, credential references, triggers, budgets, and actor authority. Any drift rejects start without creating an instance or run. Exact existing content may be treated idempotently; same-name different-content conflicts require a new operator-selected name or explicit ordinary edit/save flow rather than overwrite.

## Built-in Initialization

Built-in bundles remain repository-owned files and are embedded or generated from one reviewed source of truth. Fresh initialization creates the workflow-template root and each file with exclusive create semantics. Existing config, node-template, workflow-template, role, or support-asset paths are never truncated or replaced. A partial initialization failure reports created paths and leaves pre-existing bytes untouched; implementation must define and test whether newly created files are rolled back or retained, without claiming atomic filesystem transactions.

## Compatibility and Claim Boundaries

- Existing `Ctrl-N` node templates remain shell-only and preserve parameter behavior.
- Existing workflow import/export, stored workflow opening, headless runs, and orchestration commands remain authoritative.
- Template bundles are Lattice-native; n8n import/export does not infer or claim equivalent bundle/orchestration semantics.
- Loading proves local syntax, structure, identity, and admission only. It does not prove objective completion, agent truth, external effect uniqueness, provider usage truth, arbitrary crash safety, or production readiness.

## Portfolio Search Decision

| Family | Mechanism | Decision |
|---|---|---|
| Widen node templates | Add agent/workflow fields to the current shell template union | Rejected: conflates one-node command rendering with workflow/orchestration authority and compatibility. |
| Opaque shell loop | Make a shell node invoke Steel or Pi repeatedly | Rejected: hides rounds, budgets, approvals, recovery, and evidence from Lattice. |
| Remote example marketplace | Fetch versioned templates from a registry | Rejected for this slice: introduces network, trust, signature, update, and availability policy. |
| Local typed workflow bundles | Validate BLAKE3-bound local files, preview without effects, then reuse existing save/start admission | Selected: smallest mechanism that preserves identity and authority boundaries. |

The surviving design was adversarially checked against malformed Nickel, extra/missing files, path and symlink escape, content drift, dirty-canvas loss, name conflict, stale orchestration revision, selection-triggered execution, missing authority, and partial initialization. Each route fails before the corresponding mutation/effect boundary.

## Risks / Trade-offs

- Whole bundles add more schema and UI state than a shell template, but avoid hidden controllers.
- Exact byte identities make edits explicit and require regeneration after reviewed workflow changes.
- Local bundle assets still become authority when an operator saves/runs them; preview and admission cannot make arbitrary commands safe or sandboxed.
- Filesystem creation cannot generally be atomic across all files; exclusive-create behavior and explicit partial-result reporting are required.
- Rich template parameterization is deferred until it can bind typed workflow fields without raw source substitution or command injection.
