# Captured workflow source revisions

Lattice retains bounded author inputs separately from the executable materialized
workflow. The selected `nickel-eval.captured-modules.v1` profile and Choregraph
construction/v3 projection add finite native compilation to the existing
source-draft, admission, fork and export routes for issues #20/#21. Lattice owns
captured inputs, concrete shell registry/configuration, runtime effects and
durable admission; the producer APIs own native evaluation/elaboration.

Cargo and Nix pin the published producer cohorts that provide these APIs:
nickel-eval `f0ea0d2a31d3482ddf4e575482f6ddf7a7e7d4e9` (captured-module
evaluator) and Choregraph `fcbae0aa17bc0eab1345529adb9ff104a46540ef`
(construction/v3 projection), both acquired anonymously from their canonical
Radicle seeds. No sibling path, private override or runtime fallback selects them.

Rust callers use the producer's responsibility-qualified APIs: construction data
under `choregraph_nickel::construction::model`, compilation bindings under
`construction::association`, and current-registry readmission under
`construction::admission`. Restricted lowering uses its separate `model`, `lower`,
`identity` and `diagnostic` owners. No flat alias bridge changes the selected
canonical cohort.

## What is retained

`Workflow.authoring` is optional. Legacy workflows remain association-free; no
source, template or instance is inferred for them. Associated workflows retain:

- exact entry/member bytes in the existing workflow-template content-identity
  type and byte/path bounds: at most 64 files, 1 MiB per file and 4 MiB total;
- explicit parameter overrides (at most 64 names and 1 MiB encoded), program kind
  and the identity of this captured input revision;
- the selected exact captured-module evaluator artifact, observed closure/binding,
  library/contracts, construction registry/implementation, lowerer and profile;
- the exact observed evaluator output, native projection/binding/origins and
  materialized workflow identity at compilation, including port correspondence;
- an explicit association state, plus a durable catalog CAS token on the
  containing workflow. The workflow and association publish in one transaction.

The input record reuses bundle capture/identities; it is not another graph IR,
construction evaluator, scheduler, authority policy or origin-receipt system.
Bundle activation retains the already captured member bytes inside its workflow
association. Exact-preview activation checks remain in place; a racing creator
cannot overwrite the workflow, and same-graph/different-source is not mistaken
for the same captured association.

Admitted source associations hold `Arc<CapturedProgram>`: workflow/run/TUI
snapshot clones share immutable captured bytes rather than copying the bundle.
Editable drafts own their mutable program independently; serialized store and
archive content is unchanged by shared ownership.

Ordinary Nickel file import keeps its existing evaluator/file-context semantics.
When bounded capture succeeds it retains entry and resolved prompt-asset bytes
with `RetainedEntrySnapshot`; these are retained inputs, not proof that the file
API evaluated these exact bytes or a complete import closure. Failed capture
does not break compatible materialized import.

Captured source uses the selected isolated worker with an explicit bounded
logical module map, including exact Choregraph library/contracts for construction.
Parameterized source first evaluates declared defaults, then renders a literal
application module from those defaults and explicit overrides. The worker returns
declared/observed closure, output bytes, selector policy and cohort binding.
This binding proves request/response correspondence, **not** that the worker
actually ran: exact source readmission repeats bounded worker evaluation and
compares the complete request (entry, every module and application byte) and
response, including worker artifact. Construction execution repeats that check
before rebuilding the current concrete registry projection and comparing its
binding and materialized graph. A generic save on a new workflow name cannot
publish a caller-supplied `CapturedModules` association, even with matching
hashes. An uncaptured selected import fails closed rather than using an ambient
path; comments and literal strings containing `import` are not imports. Ordinary
file-context imports and the restricted Nickel CLI gate retain their own behavior.

The first local construction profile requires Linux and canonical regular
executable Nix-store paths for the running runtime, shell backend and captured
worker. Each selected store entry and descendant must be root-owned and
non-writable. The root-owned Nix root must not be group/other writable; a
root-owned sticky `/nix/store` namespace (including normal multi-user mode
`01775`) is accepted. Missing, mutable or oversized executables fail closed;
no same-named PATH fallback is permitted. Executable identity uses bounded
artifact bytes; build-generated source-cohort IDs remain declarations, not
executable attestations.

The operator may select `LATTICE_CONSTRUCTION_SHELL` and
`LATTICE_CLOSED_EVAL_WORKER`. Packaged defaults are
`LATTICE_CONSTRUCTION_SHELL_DEFAULT` (the exact Nix-store Bash executable) and
`LATTICE_CLOSED_EVAL_WORKER_DEFAULT` (the shipped companion worker). Changing
either selected artifact or the running runtime invalidates the retained v3
runtime binding and its exact effect-authority inspection.

Parameter values are rendered through the selected native Nickel value printer,
which preserves import/interpolation-like strings as literal data. Expanded
Nickel string fields use that same native literal escaping; `%{...}` inside an
override is not evaluated as source code.

## Source editing and parameter convention

The real command route is `lattice authoring`. It never runs a workflow, resolves
a credential provider or grants authority. Flat programs use existing portable
decoding and admission. Captured construction programs instead use the selected
bounded evaluator, native Choregraph elaboration and the host shell registry;
their output is still only a graph candidate, not execution permission.

A parameterized program evaluates to a record with `parameters` (JSON-exportable
defaults) and `build` (a Nickel function receiving selected overrides). Use
`--parameterized` for this convention and `--construction` for a native
construction candidate; both flags together select a parameterized construction.
Override names must occur in the declaration. Helpers/templates, per-instance
parameters and generated collections behave according to source recompilation;
labels are not reverse-compiled into source. Parameters grant no runtime authority.

An effect-free editing cycle, using captured `local-shell-construction.ncl` and
the exact shipped construction library/contracts, is:

```sh
lattice authoring draft local-shell-demo edit-1 --input examples/construction/local-shell-construction.ncl --construction
lattice authoring preview edit-1 --expanded-output preview.ncl
# Inspect preview.ncl before admission. No workflow has run.
lattice authoring admit edit-1
lattice authoring inspect local-shell-demo
lattice export local-shell-demo author.author.json
lattice export local-shell-demo expanded.ncl --expanded
```

`draft NAME ID` without `--input` copies an admitted source snapshot. For a new
workflow, NAME must equal the source's workflow name. `--member relative/path`
captures a selected file beneath the input directory; captured construction
imports resolve only from that map and the exact shipped library/contracts.
`--archive --input file.author.json` restores source inputs for new compilation,
not an old executable admission or effect grant.

`set-file ID MEMBER INPUT` replaces an existing captured member.
`set-parameters ID INPUT.json` replaces the override record. Both invalidate the
previous preview. A new `preview` persists either the candidate or a bounded
category-only diagnostic; raw evaluator errors/source/host paths are not put in
the diagnostic. The last admitted workflow stays unchanged on evaluation,
validation or admission failure. The failed draft remains reopenable/editable.

Preview prints a bounded node/edge/config-change summary; explicit
`--expanded-output` permits inspecting full candidate configuration. Expanded
output includes opaque credential references, **not delivered secret values**.
Treat source and configuration exports as potentially private author content.
There is no automatic redaction of arbitrary literals in an explicitly requested
source export.

In the TUI, Ctrl-W's typed authoring list includes captured source members,
parameter overrides, an explicit `Admit exact source preview` row and unavailable-attribution
information. Editing source rows compiles a **separate durable draft**, displays
a bounded diff summary, and leaves the admitted graph unchanged. Type `admit` in
the admission row only after inspection; the CLI can export the same TUI draft
for full configuration review. TUI draft IDs are `tui-` followed by the BLAKE3 of
the workflow name. Failed source text is reloaded when reopening the editor.
Save/discard materialized changes before starting a source edit; source editing
never implicitly inverses those changes.

### Machine owner command: `authoring source-edit`

Run `lattice --config CONFIG authoring source-edit`. Write **one JSON object**
to stdin and close it. On success, stdout contains one newline-terminated JSON
object and the process exits 0. Request fields are closed; unknown fields and
missing required fields are errors. The request limit is 4,210,688 encoded
bytes including whitespace; `bytes` is a JSON array of integers `0..255`, at
most 1,048,576 elements. Existing captured-program limits still apply.

Every request and response uses `schema: "lattice.source-edit.v1"`. In these
schemas all revision and artifact values are opaque strings of 64 lowercase
hex digits, except the explicitly nullable fields:

| `action` | Required request fields besides `schema` and `action` | Successful response fields besides `schema` |
| --- | --- | --- |
| `inspect` | `workflow` | `status: "inspected"`, `workflow`, `head`, `source` |
| `view` | `workflow`, `expected_head`, `draft_id`, `member` | `status: "viewed"`, `workflow`, `head`, `draft_id`, `draft_revision` (null if absent), `source_revision`, `member`, `bytes` |
| `save` | `workflow`, `expected_head`, `draft_id`, `member`, `expected_draft` (explicitly null to create), `bytes` | `status: "saved"`, `workflow`, `head`, `draft_id`, `draft_revision`, `source_revision`, `member`, `receipt` |

`inspect.source` is null for an unassociated workflow; otherwise it contains
`source_revision`, `state` (`CapturedSnapshot`, `CapturedModules`,
`RetainedEntrySnapshot` or `MaterializedFork`), `entry`, and the sorted logical
paths in `members`. `head` is the catalog CAS token, **not** an archive's source
revision. Pass this exact value as `expected_head`; the owner never selects a
newer head for you. `draft_id` is a caller-selected, store-global identifier of
1–128 ASCII letters, digits, hyphens or underscores. An existing ID must belong
to the same workflow and head, even when the caller requests creation. Choose a
different ID after a head advance. `member` must be a safe logical path already
present in the selected captured program; it is never opened on the filesystem.
When no draft exists, `view` reads from the current association; otherwise it
reads the exact draft, including any changes made by other authoring routes.
View emits the selected bytes without redaction; treat them as private content.

Pass the `draft_revision` observed by `view` as `expected_draft`, including an
explicit null if there was no draft. The token fingerprints the complete draft
including its preview and diagnostic, not just the selected member; identical
record contents produce the same token. Save replaces only that member. It
keeps the other files, kind and overrides, compiles and validates the exact
program natively, then compares both expected tokens in the same redb write
transaction. A successful write changes only the draft, including its native
preview. The `receipt` has `compiler` (the native compiler object),
`evaluated_output` (exact evaluation output identity) and `expanded_artifact`
(exact materialized candidate identity); the surrounding response binds the
workflow, head, draft, source revision and member. The compiler object has
`evaluator_source`, `evaluator_library`, `evaluator_core`, `dependency_lock`,
`lowerer_adapter`, `source_adapter`, `profile` and `worker_artifact`. This is not
a signature, publication, effect authorization or evidence of a workflow run.

When the owner rejects a request, stdout is
`{"schema":"lattice.source-edit.v1","status":"rejected","code":"…"}` and the
exit status is 1. Codes are `invalid_request`, `request_too_large`,
`unsupported_schema`, `workflow_not_found` (inspect), `source_unavailable`,
`head_conflict`, `draft_conflict`, `draft_target_mismatch`, `member_not_found`,
`validation_failed`, `store_error` and `commit_unknown`. Failure text does not
contain source or evaluator details. CLI/configuration and stdout failures may
not emit JSON. A failed validation does not replace an existing draft with
invalid text, unlike the interactive preview route. A missing reply or
`commit_unknown` is not proof of rollback: inspect and view before making a
fresh edit decision. This command cannot admit the preview or run a workload.

For focused verification, use the pinned dev shell; the host qualification app
also runs the ignored native-receipt test with the packaged binary and worker:

```sh
nix develop -c cargo test --locked --lib source_edit_commit
nix develop -c cargo test --locked --test test_source_authoring source_edit_owner
nix run .#construction-runtime-qualification -- NEW_OUTPUT_DIRECTORY
```

## Durable admission and explicit materialized forks

Source previews bind exact captured input, compiler and candidate identities.
Admission compares the submitted preview against its durable draft and the
expected workflow revision in the same redb write transaction. Changed input,
changed compiler, changed preview graph, a replaced durable draft, concurrent
admission or a stale workflow token is rejected. Per-workflow durable generation
counters prevent delete/recreate ABA from reusing a prior revision token.

TUI saves use the same expected-revision CAS. Associated workflow saves cannot
be overwritten by a source-free legacy upsert. A generic new-name save cannot
publish captured-module provenance; source preview/readmission owns that path.
For existing associations, observation and construction receipt fields are as
immutable under ordinary CAS as source, compiler and expanded identity. Only an
exact source readmission can replace them. Legacy no-source callers retain their
upsert compatibility without gaining source claims. A failed admission retains
the draft and a bounded category, without overwriting newer diagnostics.

Every materialized edit/save/export route uses
`authoring::reconcile_materialized`: a difference from the compiled materialized
artifact changes the state to `MaterializedFork`, retaining the exact original
source lineage. Typed node/workflow/operation edits, graph gestures, dirty-state
updates and catalog publication use this rule. Reverting graph content does not
silently reassert that the source reproduces a materialized fork. No source is
rewritten. Arbitrary direct public-field changes are reconciled at save/export.

In-flight workflow clones retain their old association and catalog token; source
readmission replaces the catalog object, not the existing clone. This adds no
runtime dispatch, retry, credential, authority, cancellation or budget path.
A host-qualified regression admits a newer source revision while a run of the
older revision is blocked on its first effect; that run completes the old graph,
and the old exact grant does not authorize the newer revision. Constructed runs
use the existing snapshot cancellation, shell-result validation and
orchestration budget routes.

## Export kinds and identities

Default export of an associated workflow is a `lattice.author-source.v1` JSON
archive, and requires the **`.author.json`** suffix. It preserves exact
module/function/helper bytes and parameters, plus association status; it is not
a flat Nickel program disguised with `.ncl`. A materialized fork's archive
preserves its old source lineage and explicitly says `MaterializedFork`; use
expanded export to retain the edited materialized result.
The archive reader and exporter share a 64 MiB (67,108,864-byte) serialized
representation limit, including both source and observation byte arrays, receipt
and framing. Capture itself remains at most 64 files, 1 MiB per file and 4 MiB
total; every successfully emitted archive is readable under the same cap. An
archive restores editable source, never an executable admission.


`--expanded` writes flat Nickel with a `lattice.expanded-workflow.v1` artifact
header. For legacy no-source workflows this remains the default. It is a
separately identified interchange artifact, not an author-program replacement.
A source archive reimport compiles anew; it cannot revive an earlier approval,
execution identity or compiler association by copying metadata.

These identities are deliberately distinct:

| Fact | Meaning / limitation |
|---|---|
| Source revision | Captured path/byte/program-kind/explicit-parameter identity; changing an unused retained member still changes it. |
| Materialized artifact | Exact serialized workflow semantics/configuration/metadata, with authoring and catalog token excluded; includes resolved prompt text and allocator state. It is not a normalized logical graph or execution identity. |
| Catalog revision | Durable storage admission/CAS identity and generation, atomically associated with the source. Not a runtime implementation identity. |
| Compiler association | Exact selected evaluator/cohort/observed-module and construction materials for captured construction; incomplete retained snapshots and legacy flat source are distinguished explicitly. Current host implementation is revalidated on execution. |
| Logical / execution identity | Native construction binding and current v3 admission remain separate from source or catalog identity; effects still require a fresh exact run-scoped authority grant. |
| Semantic equivalence | Equal source/output hashes do not prove whole-workflow behavioral equivalence. Equal materialized values in a bounded test are only that comparison. |

No labels, coordinates, port names or config fields are silently removed from
materialized identity. Port labels and references can participate in runtime
binding; registry names identify registrations, **not executable content
digests**. Diagnostic display names do not supply executable identity. This
feature does not invent a new logical-graph canonicalizer that declares such
fields universally cosmetic.

## Scope and qualification

The finite construction profile initially supports the real local shell runtime:
`shell-source` (no input) and `shell-stdin` (string stdin), string stdout/stderr,
exitcode output, and Unit/zero-data control dependencies. Typed signatures,
copy/drop rules, effect and invocation slot facts come from the selected host
registry, not Nickel labels. Ordinary materialized workflows retain their old
v1/v2 import/export and named-child behavior; explicit construction forks are
re-admitted as v3 without claiming source reproduction. Absent construction and
control facts are never serialized: legacy stored, portable and runner-plan
bytes, compiler identities and expanded `| Workflow` exports stay identical to
the committed format, so earlier readers that deny unknown fields still accept
them. Runtime topology changes must be newly admitted revisions, not mutations
of running snapshots.

Native origins describe scoped template/instance/node/port relationships; source
navigation uses a captured logical module and a validated byte span only where
the producer provides one. Other locations report unavailable attribution. A
graph edit never synthesizes an inverse Nickel edit: materialized forks retain
lineage but explicitly cease to claim that unchanged source reproduces them.
Source capture, preview, navigation and export do not start workloads or deliver
credentials. An operator must select a separate exact authority grant at run time.

This is authored consumer implementation, not a release receipt. Every test
that needs the captured worker or the v3 runtime is `#[ignore]`d with a reason:
the construction scenarios in `tests/test_construction_authoring.rs` and
`tests/test_construction_qualification.rs`, the v3 cases in
`tests/test_construction_runtime_binding.rs` and `tests/test_workflow_format.rs`,
every captured-preview case in `tests/test_source_authoring.rs`, and the TUI
source-origin case. Plain Cargo and the sandboxed `nextest` check run only the
non-ignored partition, which includes a regression proving that captured
evaluation fails closed when no worker is selected. The host-run qualification
app below runs the ignored partition. No general
semantic-equivalence proof, arbitrary operation pack, inverse source compiler,
or post-result mutation of an in-flight graph is provided; an in-flight run keeps
executing the exact snapshot it loaded while a newer source revision is admitted.

## Integration verification

The earlier flat-source focused targets and their observations remain in the
[focused integration record](../.cairn/changes/preserve-captured-workflow-source/evidence/focused-observation.json).
They are not a passing receipt for the new captured construction profile.
Useful narrow entrypoints after selecting the exact producer artifacts include:

```sh
nix build .#checks.x86_64-linux.nextest --no-link -L
nix run .#construction-runtime-qualification -- ./construction-qualification-output
nix develop -c cargo test --locked --test test_workflow_format
nix develop -c cargo test --locked workflow_template
nix develop -c cargo test --locked tui::authoring
nix develop -c cargo test --locked tui::app::tests::lifecycle
```

The v3 runtime admits only root-owned, non-writable Nix-store executables. A Nix
build sandbox does not map host root, so store entries appear owned by the
overflow user there and every sandboxed v3 positive fails closed. Qualification
is therefore a host-run flake app, not a flake check. `packages.<system>.construction-runtime-tests`
installs the release test executables into an immutable store output. The
`construction-runtime-qualification` app (Linux only) then runs, on the host:

- the real `lattice` binary: `import --construction` of
  `examples/construction/local-shell-construction.ncl`, `authoring inspect`,
  `authoring origin` for every node, author-source and `--expanded` export,
  `authority inspect`, and `run` with an exact grant derived from that inspection;
- negative controls: unlabeled `.ncl` source export, `run` without authority,
  `run` with a grant for another revision, and wrong-scope-handle and
  generation-cap-overflow imports that must leave the admitted revision unchanged;
- every ignored construction, runtime-binding, qualification and source-authoring
  test with the packaged companion worker and the reviewed Bash backend.

It requires one new output directory and refuses an existing path. The
directory retains `source/` (the captured example and its `.author.json`
archive), `expanded/` (the flat expanded graph), `commands.tsv` (each exact
command, expected outcome and observed exit code), `logs/` (each step's stdout
and stderr), `work/` (the store, configurations and grants used) and
`summary.json`. The output is bounded host evidence for the selected store
artifacts; it is not a semantic-equivalence, external-service, producer or
release claim. In-build Cargo/nextest binaries do not meet the v3 runtime
artifact policy and are not a qualification route.

The source regressions cover exact helper/parameter retention through store
reopen/archive; source-defined instance/shared-helper changes; failed/stale and
concurrent drafts; replaced/tampered previews; compiler drift; deletion ABA;
materialized forks; legacy imports/expanded compatibility; retained old clones;
non-execution with inert shell commands and unresolved credential references;
TUI failed-draft reopen and explicit admission; and actual registered CLI
commands/default archive versus explicit expanded export. These earlier
observations do not qualify new captured construction or lifecycle acceptance.
