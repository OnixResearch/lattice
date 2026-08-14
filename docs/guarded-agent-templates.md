# Guarded agent workflow templates

Lattice includes two local workflow-template bundles for visible, bounded coding-agent rounds:

- `cairn-agent-drain` completes one selected active Cairn change.
- `octet-agent-burndown` makes one candidate refactor against a no-disabled Octet baseline.

Each bundle contains a typed Nickel workflow, orchestration definition, role prompt, manifest, and exact BLAKE3 bindings. The bundle loader rejects changed bytes, undeclared files, unsafe paths, symbolic links, incompatible node kinds, and invalid authority shapes.

## Round model

Each round is one immutable acyclic workflow. It has discovery or a probe, one agent node, validation, post-measurement, and a pure progress decision.

The admission policy requires exactly one Pi agent node. It rejects fallback models, a second agent, nested subworkflows, triggers, credentials, retries, unlisted commands, and expanded per-round agent budgets.

A successful decrease emits an inert successor fact. It does not start another round. The operator must review and approve an exact orchestration proposal through the existing revision, authority, and budget checks.

Equality, regression, malformed data, duplicate facts, validation failure, retained Cairn selection, and Octet restoration failure block the round. A blocked progress command exits unsuccessfully. Lattice does not retry or roll back the working tree.

## Positive budgets

Both orchestration definitions enforce these limits:

| Dimension | Total limit | Per-round upper bound |
|---|---:|---:|
| Rounds | 8 | 1 |
| Agent invocations | 8 | 1 |
| Tokens | 262144 | 32768 |
| Duration | 7200000 ms | 900000 ms |

Monetary microunits are not enabled. The bundles have no schedule or webhook trigger and no automatic retry.

## Cairn agent drain

The fixed round commands are:

```text
lattice guarded cairn-discover --root .
cairn validate --root . --strict
lattice guarded cairn-discover --root .
lattice guarded decide --kind cairn --minimum-delta 1
```

Discovery opens `.cairn/changes` under the selected repository root. It rejects symbolic links, invalid identifiers, duplicates, and more than 64 active changes. It sorts the change identifiers and selects one requested identifier or the first identifier.

An empty set does not select the agent branch. A work round invokes one Pi agent with the selected snapshot. Progress requires successful validation, removal of the selected change, and an active-count decrease of at least one.

The agent prompt permits repository edits, tests, synchronization, and archive work. It permits a local commit only when repository policy requires one. It does not permit push, force-push, pull-request work, self-approval, or budget changes.

## Octet agent burn-down

The fixed round commands are:

```text
lattice guarded octet-probe --root . --artifact-dir target/lattice-guarded/octet-before --timeout-secs 600
scripts/check.rs
lattice guarded octet-probe --root . --artifact-dir target/lattice-guarded/octet-after --timeout-secs 600
lattice guarded decide --kind octet --minimum-delta 1
```

Each probe captures the exact `dylint.toml` bytes and BLAKE3 identity. It creates an owned backup in the probe artifact directory. It then installs the fixed no-disabled policy and runs `cargo octet check` through bounded process execution.

The adapter requires the temporary policy to remain exact before restoration. It restores the captured bytes atomically and verifies their identity before it reports findings. Process failure and malformed output still restore the observed exact temporary policy.

If the temporary policy changes unexpectedly, the adapter does not overwrite it. It keeps the backup and blocks for operator inspection. A pre-existing backup or temporary replacement file also blocks before mutation.

An abrupt host death can occur between replacement and restoration. The bundle does not claim arbitrary kill safety. Inspect the owned backup and current policy bytes before any later agent dispatch.

Progress requires successful repository validation, exact restoration for both probes, and a findings decrease of at least one. Flat or regressing edits remain for operator inspection. Lattice does not roll them back, commit them, or push them.

## Operator procedure

1. Initialize or copy the built-in bundles into the local `workflow-templates/` catalog.
2. Press `Ctrl-P` and select **Browse Workflow Templates**.
3. Select one guarded bundle and inspect its root, commands, Pi target, prompt, effects, budgets, notes, and non-claims.
4. Save the exact draft explicitly.
5. Start the exact orchestration explicitly with an admitted local operator identity.
6. Inspect the bounded run and progress evidence.
7. If progress improved, create and approve an exact successor proposal as a separate action.
8. If the result is complete or blocked, do not submit a successor proposal.

Preview and cancellation do not save, reserve budgets, dispatch agents, retrieve credentials, create runs, or start orchestration.

## Regeneration

Regenerate the typed workflow and orchestration files after a reviewed prompt or command change:

```text
cargo run --example generate_guarded_templates
```

Then calculate each changed file identity with:

```text
nix hash file --type blake3 --base16 PATH
```

Update only the matching manifest binding. A changed prompt, command, workflow, orchestration, budget, or reference creates a different reviewed bundle.

## Design references

The implementation reviewed `../aspen/scripts/octet-burndown.scm` at BLAKE3 `791fba3ab3c78ededa9406b57a3d90163984541d453f183f632d8326e3aa5322`. It ports strict count reduction and exact observed policy restoration.

It also reviewed `../../mc/scripts/ralph-drain-cairns.scm` from mc commit `7f36751aa4aadcbb048442c06dfbcf2addec4aaa`. That file has BLAKE3 `94ff7d3c3c527d272782c1bbd84b79da557fc6392af9dfbbaf7a21cb03f703b9`. It ports bounded sorted selection and one sequential agent per selected change.

The scripts are design references only. The bundles do not read sibling paths at runtime, follow later sibling revisions, embed either script, or claim semantic equivalence.

## Non-claims

A content identity proves exact local bytes only. It does not prove behavior, safety, correctness, authority, exactly-once effects, provider behavior, archive quality, or release readiness.

Cairn active-count reduction is a lifecycle observation. Octet finding reduction is a lint observation. Both require operator review and the repository's independent evidence gates.
