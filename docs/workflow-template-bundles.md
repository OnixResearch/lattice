# Local workflow-template bundles

Lattice loads workflow templates from the `workflow-templates/` directory next to `config.ncl`.

A bundle is separate from a shell node template. A bundle can contain a complete workflow, one optional orchestration definition, and declared support assets.

## Initialize the built-in catalog

Use a new configuration path:

```text
lattice --config ./demo/config.ncl --init
```

Initialization uses create-only file operations. It creates the config, shell node templates, and built-in workflow-template files.

If any target file exists, initialization stops before it writes a file. It does not replace operator bytes.

A race or I/O failure can occur after some new files are created. The error reports the failed relative path and the number of created files. Lattice does not claim an atomic multi-file transaction.

## Bundle layout

Each immediate bundle directory has this layout:

```text
workflow-templates/<bundle-id>/
  manifest.ncl
  workflow.ncl
  orchestration.ncl     # optional
  assets/...            # optional and declared
```

`manifest.ncl` uses a typed Nickel contract. It declares these values:

- stable bundle metadata;
- the workflow path and BLAKE3 identity;
- one optional orchestration path and BLAKE3 identity;
- every support asset path, media kind, BLAKE3 identity, and required state;
- workflow, orchestration, node, runtime, and feature compatibility;
- safe operator notes and non-claims.

The authoring contract is [`policy/workflow-template-bundle.ncl`](../policy/workflow-template-bundle.ncl).

The loader accepts local regular files only. It rejects absolute paths, parent traversal, symlinks, non-regular files, undeclared files, missing files, identity drift, and files outside named limits.

The loader does not retrieve network content. It does not execute a workflow or asset.

One invalid bundle produces one safe diagnostic. Other valid bundles remain available.

## Regenerate reviewed identities

After you review a file change, calculate its lowercase BLAKE3 identity:

```text
nix hash file --type blake3 --base16 workflow.ncl
```

Copy the exact output into the matching manifest binding. Repeat this step for the orchestration file and each declared asset.

The loader rejects stale, uppercase, prefixed, malformed, or unsupported identity values. Recalculate an identity only after you inspect the changed bytes.

## Preview in the TUI

1. Press `Ctrl-P`.
2. Select **Browse Workflow Templates**.
3. Use the arrow keys to select a bundle.
4. Press `Enter` to preview it.
5. Press `Esc` to restore the prior workflow.
6. Press `Enter` again to retain the preview as an unsaved draft.
7. Use the normal explicit save action only after you inspect the workflow.

Preview does not write the store. It does not create an orchestration instance, run, budget reservation, credential lookup, agent call, or node effect.

If the current canvas has unsaved changes, Lattice refuses replacement. Save or discard those changes before preview.

## Save and start boundaries

A saved workflow must have the exact previewed revision. A same-name workflow with different content is a conflict, not overwrite authority.

An optional orchestration remains an inert draft after preview and workflow save. A product integration can start it only through the workflow-template activation port.

The integration must call `start_orchestration_draft_with_execution` with exact admission facts, an admitted actor, and an authorized `ProductExecution`. The port records the exact bundle, workflow, orchestration, policy, and actor identities before dispatch.

The ordinary `orchestration start` CLI remains a separate route. It does not establish workflow-template origin or a bundle activation receipt.

Activation admission rechecks the saved workflow revision, policy, actor, runtime, effects, credentials, triggers, and positive budgets. Drift or a missing fact stops before instance creation or dispatch.

Selecting, previewing, or saving a template cannot start orchestration or approve a proposal.

## Identity and claim boundary

BLAKE3 identities prove the exact local bytes used for validation. They do not prove workflow behavior, command safety, agent correctness, provider truth, external effect uniqueness, crash safety, release eligibility, or production readiness.

Operators grant authority when they save and run bundle content. Local loading and preview are not a sandbox.
