# Captured workflow source revisions

Lattice retains bounded author inputs separately from the executable materialized
workflow. This implements the local source-draft, admission, fork and export
portion of [issue #21](https://github.com/OnixResearch/lattice/issues/21). It does
**not** close that issue or implement the missing Choregraph construction API.
The maintenance owner is Lattice authoring/store; graph construction/origins stay
with Choregraph and evaluation/import closure stays with nickel-eval.

## What is retained

`Workflow.authoring` is optional. Legacy workflows remain association-free; no
source, template or instance is inferred for them. Associated workflows retain:

- exact entry/member bytes in the existing workflow-template content-identity
  type and byte/path bounds: at most 64 files, 1 MiB per file and 4 MiB total;
- explicit parameter overrides (at most 64 names and 1 MiB encoded), program kind
  and the identity of this captured input revision;
- the selected evaluator source/library/core cohort, locked dependency material,
  Lattice lowerer adapter and evaluation-profile identity;
- the observed evaluator JSON identity and the exact materialized workflow
  identity at compilation, including resolved prompt contents;
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

Ordinary Nickel import keeps its existing evaluator/file-context semantics. When
bounded capture succeeds it retains entry and resolved prompt-asset bytes with
`RetainedEntrySnapshot`: these are **retained inputs**, not proof that the file
API evaluated these exact bytes or a complete import closure. If capture cannot
satisfy the bundle bounds/path policy, ordinary compatible materialized import
still succeeds without an association. Check `authoring inspect`; absence is not
reproducibility evidence.

`CapturedSnapshot` means the source-text evaluator received the captured entry
and explicit parameters and Lattice admitted the observed output. Source editing
first uses the selected native Nickel parser to reject **every syntactic import**,
including imports in dormant branches, helper functions and type annotations.
Literal strings/comments containing `import` are not import nodes. Extra captured
members may be retained assets/lineage, never substituted ambient modules. No
association is a complete-module/toolchain reproducibility receipt or safe cache
key; the supported editable program cohort is import-free entry source.

Parameter values are rendered through the selected native Nickel value printer,
which preserves import/interpolation-like strings as literal data. Expanded
Nickel string fields use that same native literal escaping; `%{...}` inside an
override is not evaluated as source code.

## Source editing and parameter convention

The real command route is `lattice authoring`. It never runs a workflow, resolves
a credential provider or grants authority. Admission uses the existing Nickel
adapter, portable decoding, Choregraph lowering and native validation.

A flat program evaluates directly to the ordinary portable workflow. A
parameterized program evaluates to a record with:

- `parameters`: a JSON-exportable record of source-declared defaults;
- `build`: a Nickel function receiving that record with the selected overrides.

Use `authoring draft ... --parameterized` for that convention. Override names
must already occur in the evaluated declaration. Lattice does not infer template
parameters from graph labels or try to recover functions from expanded nodes.
Changing a shared helper or one instance parameter does exactly what the source
program defines. Parameters do not convey runtime authority.

An effect-free editing cycle, using an already reviewed import-free author
program, is:

```sh
lattice authoring draft my-workflow edit-1 --input author.ncl --parameterized
lattice authoring preview edit-1 --expanded-output preview.ncl
# Inspect preview.ncl before admission. No workflow has run.
lattice authoring admit edit-1
lattice authoring inspect my-workflow
lattice export my-workflow author.author.json
lattice export my-workflow expanded.ncl --expanded
```

`draft NAME ID` without `--input` copies an admitted source snapshot. For a new
workflow, NAME must equal the source's workflow name. `--member relative/path`
retains an explicitly selected file beneath the input directory; it does **not**
make that file the evaluator's import resolver. `--archive --input file.author.json`
restores exact retained inputs, not execution or an old admission.

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

## Durable admission and explicit materialized forks

Source previews bind exact captured input, compiler and candidate identities.
Admission compares the submitted preview against its durable draft and the
expected workflow revision in the same redb write transaction. Changed input,
changed compiler, changed preview graph, a replaced durable draft, concurrent
admission or a stale workflow token is rejected. Per-workflow durable generation
counters prevent delete/recreate ABA from reusing a prior revision token.

TUI saves use the same expected-revision CAS. Associated workflow saves cannot
be overwritten by a source-free legacy upsert. Legacy no-source repository
callers retain their old upsert compatibility; they do not gain source claims.
A failed admission retains the draft and a bounded admission-failure category
without overwriting a newer draft's diagnostics.

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
The regression definitions check retained snapshot values without claiming a
new live-workload qualification.

## Export kinds and identities

Default export of an associated workflow is a `lattice.author-source.v1` JSON
archive, and requires the **`.author.json`** suffix. It preserves exact
module/function/helper bytes and parameters, plus association status; it is not
a flat Nickel program disguised with `.ncl`. A materialized fork's archive
preserves its old source lineage and explicitly says `MaterializedFork`; use
expanded export to retain the edited materialized result.

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
| Compiler association | Partial observed evaluator cohort/lock/adapter/profile facts, not a complete compiler executable or execution-stage identity. Not observation of ambient imports or registry executables. |
| Logical / execution identity | Remain owned by existing admission and runtime binding mechanisms; this feature does not mint substitutes. |
| Semantic equivalence | Equal source/output hashes do not prove whole-workflow behavioral equivalence. Equal materialized values in a bounded test are only that comparison. |

No labels, coordinates, port names or config fields are silently removed from
materialized identity. Port labels and references can participate in runtime
binding; registry names identify registrations, **not executable content
digests**. Diagnostic display names do not supply executable identity. This
feature does not invent a new logical-graph canonicalizer that declares such
fields universally cosmetic.

## Explicitly blocked acceptance criteria

The selected nickel-eval API accepts source/file and ambient import paths, but
has no captured-byte-map-only import resolver or complete dependency-closure
receipt. Captured member bytes are preserved, not proof of evaluated module
closure. **Source preview/readmission fails closed for syntactic imports** rather
than appearing to compile an edited module while reading an ambient old module.
Ordinary materialized file/bundle import remains explicitly retained-snapshot
compatibility, not source readmission. Complete selected-module reproducibility
and import/library closure drift invalidation remain unqualified. Ambient imports
are never promoted to captured provenance. Current native workflow validation and
Choregraph graph admission run again at readmission; a persisted preview cannot
skip the current host's validation policy.

[Choregraph #6](https://github.com/OnixResearch/choregraph/issues/6) remains the
producer prerequisite for construction and bounded template/instance/port
origin mappings. The selected public Choregraph API exposes lowering and
caller-bound translation identity materials, **not** construction origins or
supported captured logical source spans. Node/port attribution therefore reports
unavailable. There are no fabricated translation receipts, inferred origins,
inverse compiler, hygienic-instance claims or generated-node source navigation.

Registry implementation drift and execution-stage binding remain with #20 and
the existing exact runtime owners; registry-name equality is insufficient.
Tests do not qualify complete-module origins, complete closure, general shared
handle semantics, logical equivalence or live execution provenance. Issue #21's
full compositional import/origin-navigation/drift/four-identity-equivalence
acceptance remains open even when the local source-edit/store regressions pass.

## Integration verification

The integrated focused targets passed. Their exact invocation and per-target
observations are retained in the [focused integration record](../.cairn/changes/preserve-captured-workflow-source/evidence/focused-observation.json).
The following narrower entrypoints remain useful for reproducing individual paths:

```sh
nix develop -c cargo test --locked --test test_source_authoring
nix develop -c cargo test --locked --test test_workflow_format
nix develop -c cargo test --locked workflow_template
nix develop -c cargo test --locked tui::authoring
nix develop -c cargo test --locked tui::app::tests::lifecycle
```

The source regressions cover exact helper/parameter retention through store
reopen/archive; source-defined instance/shared-helper changes; failed/stale and
concurrent drafts; replaced/tampered previews; compiler drift; deletion ABA;
materialized forks; legacy imports/expanded compatibility; retained old clones;
non-execution with inert shell commands and unresolved credential references;
TUI failed-draft reopen and explicit admission; and actual registered CLI
commands/default archive versus explicit expanded export. These observations do
not qualify the blocked producer contracts above or constitute lifecycle acceptance.
