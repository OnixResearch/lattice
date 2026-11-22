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

- `schema_version = 2`;
- stable bundle metadata;
- a required `mode`, `"read_only"` or `"mutating"`;
- a required `params` list of typed parameters, which can be empty;
- the workflow path and BLAKE3 identity;
- one optional orchestration path and BLAKE3 identity;
- every support asset path, media kind, BLAKE3 identity, and required state;
- workflow, orchestration, node, runtime, and feature compatibility;
- safe operator notes and non-claims.

The manifest identity is the BLAKE3 identity of the manifest bytes, so it covers `mode` and every parameter declaration.

The authoring contract is [`policy/workflow-template-bundle.ncl`](../policy/workflow-template-bundle.ncl).

The loader accepts local regular files only. It rejects absolute paths, parent traversal, symlinks, non-regular files, undeclared files, missing files, identity drift, and files outside named limits.

The loader does not retrieve network content. It does not execute a workflow or asset.

One invalid bundle produces one safe diagnostic. Other valid bundles remain available.

## Mode

`mode` is the upper bound on what the bundle's workflow may do. The loader computes the least mode that admits every node and rejects a `read_only` bundle whose workflow needs `mutating` with `mode_violation`. Save and start planning repeat the same check.

The classification is conservative. A node counts as read-only only when it is provably so:

| Node | Class |
| --- | --- |
| Condition, merge, approval | Pure: no external effect |
| File `read`, image input | Read-only: a bounded read through an operator-configured root |
| File `write` or `append` | Mutating |
| Shell, operation, HTTP of any method, agent | Mutating |
| Subworkflow | Mutating: the child workflow is outside the bundle |

A shell command is never provably read-only, so every bundle with a shell node is `mutating`, including the built-in catalog review and shell preview. The TUI browser marks each bundle's mode, and a `mutating` bundle shows a red warning when selected.

## Parameters

Each `params` entry is a closed record:

```nickel
{
  name = "catalog_root",
  type = { kind = "relative_path", max_bytes = 256 },
  required = false,
  default = "builtins/agent-evaluations",
  bindings = [{ node = "Validate evaluation catalog", shell_env = "LATTICE_PARAM_CATALOG_ROOT" }],
}
```

- `name` is lowercase ASCII, digits, and `_`, at most 64 bytes, and unique.
- `type` is one of:
  - `{ kind = "bool" }`;
  - `{ kind = "integer", min, max }`, inclusive;
  - `{ kind = "enum", values }`, with 1 to 32 unique lowercase tokens;
  - `{ kind = "relative_path", max_bytes }`, with 1 to 256 bytes.
  No type accepts free-form text.
- A required parameter has no `default`. An optional parameter must have a `default` that satisfies its type.
- `bindings` lists 1 to 8 targets. Each names a shell node by its label, which must match exactly one node, and the environment variable it receives. Labels are used because Nickel import renumbers node identities. The key must start with `LATTICE_PARAM_`, so a parameter can never set `PATH` or another ambient variable.

A bundle declares at most 16 parameters. Unknown fields and other kinds are rejected.

A `relative_path` value uses only ASCII letters, digits, `.`, `_`, `-`, and `/`. Its components are nonempty, never `..`, and never start with `-`. An `enum` value is one of the declared tokens. These values contain no whitespace, quote, or shell metacharacter.

### Binding without shell interpolation

A value reaches the workflow only as the environment value of each declared shell node. It is never spliced into command text, and it can never choose the program.

Every shell node that receives a value must be one simple command in a strict grammar:

- words are separated by spaces;
- each word is either a literal over ASCII letters, digits, and `-`, `_`, `.`, `/`, `=`, `:`, `,`, `+`, `@`, `%`, or exactly `"${LATTICE_PARAM_NAME}"` as a whole word for a key bound to that node;
- the first word is a literal program without `=`, so it is never a reference or an environment assignment;
- the first word's basename is not an argument-executing program: `sh`, `bash`, `dash`, `zsh`, `ksh`, `fish`, `eval`, `exec`, `env`, `xargs`, `command`, `builtin`, `source`, `.`, `nohup`, `timeout`, `nice`, `ionice`, `setsid`, `stdbuf`, `taskset`, `chrt`, `sudo`, `doas`, `su`, `runuser`, `busybox`, `time`, `watch`, `parallel`, `find`, or `git`;
- no word is `-exec`, `-execdir`, `-ok`, or `-c`.

The literal alphabet excludes `$`, backticks, quotes, backslashes, `;`, `|`, `&`, `<`, `>`, `(`, `)`, `{`, `}`, `*`, `?`, `[`, `~`, `#`, whitespace other than the separating space, newline, and NUL. A bound command therefore has no command substitution, arithmetic, here-document, redirection, glob, or control operator. The shell passes each value as one argument without word splitting or globbing.

No shell node in a bundle may bind a `LATTICE_PARAM_*` key as a credential (`secret_env`), so a credential can never shadow a value at execution.

The template workflow must hold each optional parameter's rendered default under its key and must not hold a required parameter's key. Binding the defaults therefore reproduces the reviewed workflow exactly.

Non-claim: the grammar fixes the program and keeps values out of shell syntax, but a program that interprets its own arguments as code (a script interpreter, a template engine, a query language) can still turn a value into code. Reviewers must not bind parameters into such programs.

### Resolution and identity

`lattice::workflow_template::params::resolve(declared, supplied)` is the pure check for supplied values. It rejects unknown names, wrong types, out-of-bounds values, and missing required values. Its diagnostic names only a declared parameter and never echoes a supplied value. The returned `params::Resolved` holds every value sorted by name and a BLAKE3 identity over each name, value kind, and value.

`lattice::workflow_template::bind(template, supplied)` resolves values against a catalog template. It writes them into a copy of the workflow and rebinds an optional orchestration to the new workflow revision. Then it re-runs the template's admission checks: workflow validation, compatibility, mode, orchestration binding, guarded authority, and the binding grammar. The bound bundle's identity is BLAKE3 over the template identity, the resolved-value identity, and the bound workflow revision.

`ValidatedBundle` content is read-only outside Lattice; read it through `identity()`, `manifest()`, `workflow()`, `workflow_revision()`, `orchestration()`, `orchestration_revision()`, `file_identities()`, and `params()`. `save_draft`, activation planning, and start call `reverify`, which re-resolves the recorded values, checks that the workflow still holds them, repeats full content admission, and checks that the recomputed revision and identity equal the bound ones. Any difference refuses with `bound_content_drift` (`BoundContentDrift`) before a store write or dispatch.

Parameters cannot widen effect authority. Binding changes only declared environment values, so node kinds, commands, resources, and abilities stay as reviewed. `lattice authority inspect` on the saved bound workflow lists the same grants as the template, under the bound workflow revision.

### Built-in migration

All four built-in bundles use schema 2 and `mode = "mutating"`.

- `agent-evaluation-catalog-review` declares `catalog_root`, a `relative_path` that defaults to `builtins/agent-evaluations`. Its command reads it as `--root "${LATTICE_PARAM_CATALOG_ROOT}"`.
- `cairn-agent-drain` and `octet-agent-burndown` keep `--root .` fixed and declare no parameters. The guarded authority check pins the root, the exact command set, an empty environment, and the agent workspace. A path parameter there would change where an agent may edit, so it is not a sound parameter.
- `shell-preview` declares no parameters.

## Schema 1 compatibility

A schema 1 manifest has neither `mode` nor `params`. It still loads as a `mutating` bundle with no parameters. Its manifest bytes are unchanged, so its manifest and template identities are unchanged. This is the conservative reading of a manifest that declared no limit, and it keeps catalogs copied by an earlier `--init` loadable without edits.

A schema 1 manifest that names `mode` or `params`, even as `null`, fails with `unsupported_schema`; the loader checks key presence before deserializing. A schema 2 manifest without either field fails with `invalid_manifest`. Any other schema version fails with `unsupported_schema`. To declare a mode or parameters, migrate the manifest to schema 2.

## Regenerate reviewed identities

After you review a file change, calculate its lowercase BLAKE3 identity:

```text
nix hash file --type blake3 --base16 workflow.ncl
```

Copy the exact output into the matching manifest binding. Repeat this step for the orchestration file and each declared asset.

The loader rejects stale, uppercase, prefixed, malformed, or unsupported identity values. Recalculate an identity only after you inspect the changed bytes.

## Install from a runbook catalog

Bundles can also come from a git-hosted runbook catalog. An operation pack carries them. Use `lattice pack catalog-fetch` to retrieve an exact commit, then `lattice pack lock`. Use `lattice pack catalog-install --catalog <config directory>/workflow-templates` to admit the bundles into this directory. Each bundle is verified with this loader and installed create-only. Installation never replaces an existing bundle directory and grants no authority.

`lattice pack catalog-list` lists this directory with the same loader. The loader itself still reads only local files. See [Runbook catalogs](operation-packs.md#runbook-catalogs).

## Preview in the TUI

1. Press `Ctrl-P`.
2. Select **Browse Workflow Templates**.
3. Use the arrow keys to select a bundle. The browser shows its mode and each parameter's default or `required`.
4. Press `Enter` to preview it.
5. Press `Esc` to restore the prior workflow.
6. Press `Enter` again to retain the preview as an unsaved draft.
7. Use the normal explicit save action only after you inspect the workflow.

The TUI supplies no parameter values. Preview binds every optional parameter to its default and shows `mode=` in its status. A bundle with a required parameter cannot be previewed in the TUI. A product integration must bind it with `bind`.

Preview does not write the store. It does not create an orchestration instance, run, budget reservation, credential lookup, agent call, or node effect.

If the current canvas has unsaved changes, Lattice refuses replacement. Save or discard those changes before preview.

## Save and start boundaries

Save and start accept only a bound bundle. `save_draft` fails with `ParamsUnbound` for an unbound catalog template, and activation planning blocks with `ParamsUnbound` or `ModeViolation`.

A saved workflow must have the exact previewed revision. A same-name workflow with different content is a conflict, not overwrite authority. This includes the same template bound to different values.

An optional orchestration remains an inert draft after preview and workflow save. A product integration can start it only through the workflow-template activation port.

The integration must call `start_orchestration_draft_with_execution` with exact admission facts, an admitted actor, and an authorized `ProductExecution`. The port records the exact bundle, workflow, orchestration, policy, and actor identities before dispatch. The recorded bundle identity is the bound identity, so it commits to the resolved values.

The ordinary `orchestration start` CLI remains a separate route. It does not establish workflow-template origin or a bundle activation receipt.

Activation admission rechecks the bound values, the mode, the saved workflow revision, policy, actor, runtime, effects, credentials, triggers, and positive budgets. Drift or a missing fact stops before instance creation or dispatch.

Selecting, previewing, or saving a template cannot start orchestration or approve a proposal.

## Identity and claim boundary

BLAKE3 identities prove the exact local bytes used for validation. They do not prove workflow behavior, command safety, agent correctness, provider truth, external effect uniqueness, crash safety, release eligibility, or production readiness.

Operators grant authority when they save and run bundle content. Local loading and preview are not a sandbox.
