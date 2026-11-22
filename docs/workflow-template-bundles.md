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
- `bindings` lists 1 to 8 targets. Each names a shell node by its label, which must match exactly one node, and the environment variable it receives. Labels are used because Nickel import renumbers node identities. Every binding target key must start with `LATTICE_PARAM_`, so a binding can never target `PATH` or another ambient variable.

A bundle declares at most 16 parameters. Unknown fields and other kinds are rejected.

A `relative_path` value uses only ASCII letters, digits, `.`, `_`, `-`, and `/`. Its components are nonempty, never `..`, and never start with `-`. An `enum` value is one of the declared tokens. These values contain no whitespace, quote, or shell metacharacter. An `integer` whose bounds admit negative numbers renders as `-N`, which a receiving program may read as an option; declare non-negative bounds unless the program expects a negative argument.

### Binding without shell interpolation

A value reaches the workflow only as the environment value of each declared shell node. In the receiving node it is never spliced into command text: it never takes the command-name position of the bound command, and the shell never parses it as shell syntax.

Every shell node that receives a value must be one simple command in a strict grammar:

- words are separated by spaces;
- each word is either a literal over ASCII letters, digits, and `-`, `_`, `.`, `/`, `=`, `:`, `,`, `+`, `@`, `%`, or exactly `"${LATTICE_PARAM_NAME}"` as a whole word for a key bound to that node;
- the first word is a literal program without `=`, so it is never a reference or an environment assignment;
- the first word's basename, compared ignoring ASCII case and a trailing version suffix for shells (`bash5`, `ksh93`, `zsh-5.9`), is neither a shell nor on the first-word deny list below;
- a shell named as a separate later word is refused, so a wrapper cannot hand a value to it (`flock l bash "${…}"`); other names stay valid as arguments, as in `systemctl enable`, `kubectl get jobs`, `find .` and `tar -xf -`. A shell chosen through an option such as `--shell=bash`, `-sbash` or `SHELL=bash`, or a program that runs an argument through a shell such as `tmux` or `hyperfine` named in a later word, still gets through; see the non-claims below;
- no word is exactly `-exec`, `-execdir`, `-ok`, `-okdir`, or `-c`.

The deny lists are defense in depth against the shell's own argument-executing builtins and common wrappers, not a guarantee:

- shells, checked in every word, ignoring a trailing `-static` and version suffix: `sh`, `bash`, `dash`, `zsh`, `ksh`, `fish`, `ash`, `mksh`, `lksh`, `rbash`, `rksh`, `oksh`, `yash`, `posh`, `csh`, `tcsh`, `pwsh`, `nu`, `elvish`, `xonsh`, `hush`, `osh`, `ysh`, `sash`;
- first word only, builtins that execute, load, or evaluate an argument: `eval`, `exec`, `command`, `builtin`, `source`, `.`, `trap`, `jobs`, `coproc`, `mapfile`, `readarray`, `enable`, `let`, `declare`, `typeset`, `local`;
- first word only, precautionary builtins and zsh precommand modifiers: `fc`, `bind`, `complete`, `compgen`, `noglob`, `nocorrect`, `-`;
- first word only, wrappers that run an argument: `env`, `tmux`, `screen`, `hyperfine`, `xargs`, `nohup`, `timeout`, `nice`, `ionice`, `setsid`, `stdbuf`, `taskset`, `chrt`, `sudo`, `doas`, `su`, `runuser`, `busybox`, `time`, `watch`, `parallel`, `flock`, `prlimit`, `setpriv`, `unshare`, `nsenter`, `strace`, `ltrace`, `script`, `systemd-run`, `firejail`, `bwrap`, `chroot`, `nix-shell`.

`git` and `find` are allowed so runbooks can run `git -C "${…}" checkout "${…}"` and `find . -name "${…}"`; `find`'s executing options are refused as words. A repository a value selects (`-C`, `--git-dir`, `--work-tree`, `--exec-path`) brings its own hooks and config, which can run code.

The receiving node's `cwd` and `env` may hold no `{{…}}` expression. Otherwise another node's output, for example `PATH={{node:A:…}}`, could change where or how that node looks up its program. Such a node fails with `param_target_expression`. Expressions also cannot appear in the bound command, because `{` and `}` are outside the literal alphabet.

The receiving node's static `env` and credential bindings may not set keys that make the shell run something before or around the command. Such a node fails with `param_target_env` (`bound_content_drift` when re-verified) when:

- an env or credential key starts with `BASH_FUNC_` or contains `%%`: bash imports it as a function, for example one that replaces `printf` and runs its argument;
- an env or credential key is `BASH_ENV`: a non-interactive bash sources the file it names;
- an env or credential key is `SSH_CLIENT`, `SSH2_CLIENT`, `SHLVL`, or `HOME`: a bash built with `SSH_SOURCE_BASHRC` sources `$HOME/.bashrc` before a `-c` command when an SSH variable is set and `SHLVL` is below 2, and `HOME` chooses that file;
- an env or credential key is `SHELLOPTS`, `BASHOPTS`, or `PS4`: they import options such as `xtrace`, and `PS4` is expanded for every traced command;
- an env value contains `$` or a backtick: it is expanded when a builtin evaluates the variable arithmetically, for example `x=a[$(…)]`.

The check is limited to these keys and characters. `ENV` is allowed because non-interactive `sh -c` and `bash -c` never read it, so `ENV=production` binds. Parameter values never contain `$` or a backtick, and ordinary static env such as `RUST_LOG=info` stays valid.

The literal alphabet excludes `$`, backticks, quotes, backslashes, `;`, `|`, `&`, `<`, `>`, `(`, `)`, `{`, `}`, `*`, `?`, `[`, `~`, `#`, whitespace other than the separating space, newline, and NUL. A bound command therefore has no command substitution, arithmetic, here-document, redirection, glob, or control operator. The shell passes each value as one argument without word splitting or globbing.

No shell node in a bundle may bind a `LATTICE_PARAM_*` key as a credential (`secret_env`), so a credential can never shadow a value at execution.

The template workflow must hold each optional parameter's rendered default under its key and must not hold a required parameter's key. Binding the defaults therefore reproduces the reviewed workflow exactly.

Non-claims:

- Any program that executes an argument can still run a program a value chooses. Examples: `git` through aliases, `--upload-pack`, `--receive-pack`, `--exec` or `ext::` URLs, and through the hooks and config of a repository a value selects with `-C`, `--git-dir`, `--work-tree` or `--exec-path`; `fd -x`; `rg --pre`; `nix develop --command`; script interpreters, template engines and query languages; a shell chosen by an option such as `--shell=bash`, `-sbash` or `SHELL=bash`; and wrappers named only in a later word or not on the list. Reviewing the bound command for such programs is the bundle reviewer's job. `lattice authority inspect` lists the bound revision's exact grants for the resolved values.
- Other nodes can consume the receiving node's outputs through expressions or stdin edges. Those nodes are ordinary reviewed workflow content; the grammar governs only receiving nodes.

### Resolution and identity

`lattice::workflow_template::params::resolve(declared, supplied)` is the pure check for supplied values. It rejects unknown names, wrong types, out-of-bounds values, and missing required values. Its diagnostic names only a declared parameter and never echoes a supplied value. The returned `params::Resolved` holds every value sorted by name and a BLAKE3 identity over each name, value kind, and value.

`lattice::workflow_template::bind(template, supplied)` resolves values against a catalog template. It writes them into a copy of the workflow and rebinds an optional orchestration to the new workflow revision. Then it re-runs the template's admission checks: workflow validation, compatibility, mode, orchestration binding, guarded authority, and the binding grammar. The bound bundle's identity is BLAKE3 over the template identity, the resolved-value identity, and the bound workflow revision. The bound workflow's source association is reconciled exactly as the store reconciles it on save: it becomes `MaterializedFork` when the values change the expanded graph and stays unchanged otherwise, so a saved bound bundle re-saves and activates as exact. The workflow revision does not cover the association, so this does not change it.

`ValidatedBundle` content is read-only outside Lattice; read it through `identity()`, `manifest()`, `workflow()`, `workflow_revision()`, `orchestration()`, `orchestration_revision()`, `file_identities()`, and `params()`. `save_draft`, activation planning, and start call `reverify` before any store write or dispatch. It checks, in order, that the recorded values re-resolve to the same values; that the workflow still holds them and passes the target checks above; that reconciling the workflow's source association leaves it unchanged; that the content passes full admission again; that the recomputed workflow revision equals the recorded and bound revisions; that the recomputed orchestration revision equals the recorded one; that the template identity recomputed from the manifest and file identities equals the bound template; and that the bound identity recomputed from template, values, and revision equals the bundle identity. Every entry point applies the same rule. An unbound bundle refuses with `ParamsUnbound`. For a bound bundle, a mode failure refuses with `mode_violation` (`ModeViolation` from `save_draft`, a `ModeViolation` conflict from `plan_application`, a `ModeViolation` block from activation planning), and any other failure refuses with `bound_content_drift` (`BoundContentDrift`).

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

Save and start accept only a bound bundle. `save_draft` fails with `ParamsUnbound` for an unbound catalog template, with `ModeViolation` when the mode no longer admits the workflow, and with `BoundContentDrift` when bound content otherwise no longer verifies. Activation planning blocks with `ParamsUnbound`, `ModeViolation`, or `BoundContentDrift`.

A saved workflow must have the exact previewed revision. A same-name workflow with different content is a conflict, not overwrite authority. This includes the same template bound to different values.

Saving a bound bundle also retains its template binding in the store, keyed by the saved workflow name: the bundle id, the template identity, the bound identity, and the saved workflow revision. `save_draft` writes it only after `reverify` passes and the plan is `CreateDraft` or `ExactAlreadyPresent`. A later save of the same name replaces it. `lattice run --input` uses this binding.

An optional orchestration remains an inert draft after preview and workflow save. A product integration can start it only through the workflow-template activation port.

The integration must call `start_orchestration_draft_with_execution` with exact admission facts, an admitted actor, and an authorized `ProductExecution`. The port records the exact bundle, workflow, orchestration, policy, and actor identities before dispatch. The recorded bundle identity is the bound identity, so it commits to the resolved values.

The ordinary `orchestration start` CLI remains a separate route. It does not establish workflow-template origin or a bundle activation receipt.

Activation admission rechecks the bound values, the mode, the saved workflow revision, policy, actor, runtime, effects, credentials, triggers, and positive budgets. Drift or a missing fact stops before instance creation or dispatch.

Selecting, previewing, or saving a template cannot start orchestration or approve a proposal.

## Run inputs

A saved workflow retained from a parameterized bundle accepts typed values per run:

```text
lattice run agent-evaluation-catalog-review --input catalog_root=fixtures/catalog
lattice run NAME --input count=3 --input-file inputs.json --output json
```

`--input NAME=VALUE` is repeatable. `--input-file PATH` holds one JSON object of at most 64 KiB, such as `{"enabled": true, "format": "json"}`. A name may appear only once across both sources, including within one source; a repeat fails with `an input name is supplied more than once`.

An `--input` value takes the declared type of its parameter. A `bool` takes exactly `true` or `false`, and an `integer` takes a canonical decimal such as `-3`, not `+3`, `03`, or `3.0`. An `enum` or `relative_path` takes the text. A JSON value keeps its JSON type, so `"true"` for a `bool` is a wrong type.

### How a saved workflow carries its parameters

The saved workflow does not copy the declarations. It retains which template it was bound from, by identity, when it is saved. A run with inputs then rebinds through that template:

1. Load the saved workflow and its retained binding. A workflow saved before bindings were retained has none and is refused with `no retained workflow-template binding; save it again from its bundle`.
2. Require the saved workflow's current revision to equal the retained revision. A workflow edited or re-saved by another route is refused with a drift diagnostic.
3. Load only the bundle directory the binding names, `workflow-templates/<bundle-id>/` next to `config.ncl`, with the catalog's single-bundle loader, and require the retained template identity. Other bundles are not read, so a catalog over its 64-directory limit still works. A bundle that fails to load is refused with its load diagnostic; a changed bundle is refused as unavailable. Re-save the workflow from the reviewed bundle to adopt a new version.
4. Refuse a template that declares no parameters. Type the values, apply the secret rule, and bind them with `workflow_template::bind`, which resolves them with `params::resolve` and repeats the template's full admission.
5. Treat the run as a start boundary: call `reverify` on the bound bundle, as save and start do, and require the recorded input identity to name exactly that bundle, revision, and values. Drift refuses with `bound_content_drift`.

`--input-file` is opened without blocking (`O_NONBLOCK` on Unix) and refused unless the opened handle is a regular file, so a FIFO or device fails at once. `lattice run` reads its inputs before it opens the store, so a refused input file never holds the store.

Inputs are a full binding of the template's declarations, not an overlay on the saved values. An omitted optional parameter takes its declared default, and an omitted required parameter fails, even if the saved workflow holds a value for it.

A workflow with an approval gate refuses inputs. Resume reloads the saved workflow revision, which a bound run does not have.

Every check above runs before Lattice reserves a run id, writes a run or input record, or runs a node. A diagnostic names at most a declared parameter. It never echoes a supplied value or an undeclared name. Admitted inputs are a value only `admit` constructs.

### Recorded identity

The run reserves its run id and records its input identity in one store transaction. The identity holds the bundle id, the template identity, the bound identity, the bound workflow revision, the `params::Resolved` value identity over every name, kind, and value, and each declared name with its kind and whether it was supplied. It holds no value. The bound workflow then runs under the saved workflow's name, and its run, detail, and replay records are written as for any run.

`runs show` adds an `inputs` object only for such a run:

```json
"inputs": {
  "template": "b3:…",
  "bound": "b3:…",
  "workflow_revision": "b3:…",
  "values": "b3:…",
  "params": [{ "name": "catalog_root", "kind": "relative_path", "supplied": true }]
}
```

It omits the workflow and bundle names, as the rest of the report does. It shows one value identity instead of per-value digests. A caller that holds the values reproduces it with `params::resolve`. The values are omitted from the output, not hidden: the shown identities are unsalted BLAKE3, so anyone with the catalog can enumerate low-entropy inputs such as a `bool`, an enum, or a bounded integer and match them. An input record that does not belong to the run makes the report invalid with `inputs_identity_mismatch`.

An input record is written with its run id reservation. If the run then fails before its run record is written, for example when its authority context cannot load, the record stays with that id. `runs show` reports the id as not found, run listings never include it, and the id is never reused.

### Authority inspection

`lattice authority inspect NAME` accepts the same `--input` and `--input-file`. It admits them as a run does, without writing state, and prints the grants of the bound workflow revision with the same `inputs` object. Inside `authority inspect`, the `inputs` identities use the report's `blake3:` prefix, so the bound workflow revision renders exactly as the top-level `workflow_revision`; `runs show` uses `b3:` throughout. Inputs never change grants, but the bound revision differs from the saved one, so a standalone authority profile written from this output authorizes exactly the inspected values. Other values need a new inspection.

### Secret-like inputs

Run inputs are never a secret channel. Credential references remain the only secret path. The rule applies only to `relative_path` parameters, the only type that carries caller text. A `bool` or bounded `integer` cannot carry a credential, and an `enum` value is one of the reviewed manifest tokens, like a default, so names such as `max_tokens` or `auth_mode` on those types are allowed. For a `relative_path` parameter, the rule refuses:

- a name, or a binding key after `LATTICE_PARAM_`, with one of these `_`-separated words, ignoring case: `token`, `tokens`, `secret`, `secrets`, `password`, `passwords`, `passwd`, `passphrase`, `credential`, `credentials`, `apikey`, `privatekey`, `auth`, `bearer`, `cookie`, `pat`, or `otp`; or a qualifier `api`, `private`, `access`, `secret`, `session`, `signing`, `client`, or `encryption` followed by `key`;
- a supplied value with a `/`-separated component in a known provider token format: the provider prefix followed by at least that format's minimum suffix, drawn only from that format's characters. Examples are `ghp_` with 30 letters and digits, `github_pat_` with 40 of letters, digits and `_`, `xoxb-` with 20 of letters, digits and `-`, `sk-proj-` with 40 base64url characters, `AKIA` with 16 uppercase letters and digits, and `pypi-` with 50 base64url characters;
- a supplied value with a run of at least 20 ASCII letters and digits between separators (`/`, `-`, `_`, `.`) that mixes uppercase, lowercase, and digits.

Refs and paths such as `dependabot/npm_and_yarn/express-4.18.2`, `.github/workflows/pypi-publish.yml`, `data/ASIA_PACIFIC_REGIONS.csv`, and `feature/PROJ-1234-add-retry-to-upload-client` are accepted. Defaults are reviewed manifest bytes and are not checked by value. A binding into a credential-bound key cannot occur, because every target key starts with `LATTICE_PARAM_` and no node may bind such a key as a credential. The rule is a pattern check. It does not prove that an accepted value is not sensitive, and it does not catch every token format.

### Runs without inputs

Without `--input` and `--input-file`, `lattice run` and `authority inspect` take the same path as before. They do not read the retained binding or the catalog and do not write an input record. The run identity, run record, summary, and report bytes are unchanged.

## Identity and claim boundary

BLAKE3 identities prove the exact local bytes used for validation. They do not prove workflow behavior, command safety, agent correctness, provider truth, external effect uniqueness, crash safety, release eligibility, or production readiness.

Operators grant authority when they save and run bundle content. Local loading and preview are not a sandbox.
