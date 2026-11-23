## Design

`Workflow` has optional source association and catalog-revision fields with serde
legacy defaults. The captured record reuses workflow-template identities and
bounds. It stores exact member bytes, entry, program kind, explicit parameter
overrides, compiler facts, evaluator-output identity, compiled materialized
identity and a truthful association state. Portable flat workflow interchange is
unchanged. No construction/origin DTO is manufactured.
The immutable association program is shared with `Arc<CapturedProgram>` so run
and TUI workflow clones do not duplicate captured bundle bytes. Editable drafts
retain owned mutable programs. JSON persistence/archive representation is the
same as an owned program.

Ordinary file import retains original file-context evaluation. Bounded retained
entry/prompt snapshots are explicitly weaker than the source-text preview route;
neither claims captured-only module resolution. Bundle capture keeps its actual
captured map, but the selected evaluator still evaluates entry text, not a new
module resolver. Capturing module bytes is retention, not observation that those
bytes were imported.
Source preview/readmission is restricted to import-free entry source by the
selected native Nickel AST parser, not string matching. Every import node is
rejected before evaluation, including dormant imports. Import-like comments and
literal strings remain valid; captured module edits cannot silently fall back to
ambient bytes. The exact existing patched parser is a normal dependency, not a
second evaluator or a sibling source import.

The parameterized source profile evaluates `program.parameters`, rejects override
names absent from that record, then evaluates `program.build` with defaults plus
overrides through selected nickel-eval. Decoding, Choregraph lowering and native
validation precede candidate construction. This does not start an executor,
construct credential providers or grant authority.
Parameter literals use native Nickel value printing, including interpolation
escaping; generated parameter applications receive AST import admission too.
Expanded string fields use the native printer as well, so imported text in a
parameter remains literal through explicit flat export and reimport.

Drafts are separate redb records. Preview failure replaces only the draft's
bounded diagnostic. Source admission compares exact stored draft/input/compiler/
candidate association and expected catalog revision in a write transaction. The
workflow, source association, new catalog generation/token and consumed draft
state publish together. Monotonic per-name generations survive deletion, so an
identical delete/recreate does not revive stale CAS authority. Failed admission
records a category only if that exact draft is still current.
Readmission also reruns current native workflow validation and Choregraph graph
admission; partial compiler input descriptors are not complete compiler or
execution-stage identities and cannot exempt a stored preview from validation.

Legacy source-free repository upsert remains compatible; it cannot replace an
associated workflow without its expected token. TUI saves always use CAS.
Typed node/workflow/operation edits, graph synchronization, dirty-state hooks and
catalog save/export invoke the same materialized-fork decision. Source bytes and
compiled artifact identity remain lineage; changed graph/configuration becomes
`MaterializedFork`. Reverts do not silently promote the association. Runtime
workflow clones keep their old captured association and catalog token.

Default associated export is a schema-tagged exact source JSON archive and
requires `.author.json`. Flat expanded Nickel is separately labeled and requires
`--expanded`; legacy source-free export remains flat by default. An archive
reimport retains bytes but recompiles/revalidates; it does not import authority,
old executable bindings or an admission result.

## Identity and producer boundaries

Source identity includes retained unused files and explicit parameters. The
materialized artifact hashes exact workflow fields (including metadata, labels,
resolved prompts and allocator state), excluding source association and catalog
CAS token. It is deliberately not a logical graph or executable digest.
Compiler facts bind selected evaluator source/library/core, lock material and
Lattice adapter/profile bytes. Runtime registry implementation binding stays
with existing #20 owners; names do not identify executable content. Equality of
these records is not semantic-equivalence proof.

Missing producer APIs: nickel-eval captured-byte-map-only import/closure receipt;
Choregraph #6 bounded construction/template-instance-port origin map and
supported captured logical source spans. No complete-module reproducibility,
import-closure drift checks, generated-node source navigation or four-identity
semantic equivalence qualification is claimed.

## Verification ownership

Focused definitions cover source retention/reopen/export, source-declared
parameters and helpers, failed/stale/concurrent/tampered drafts, compiler drift,
CAS delete/recreate ABA, forks, no-source compatibility, unchanged old clones,
effect-free compilation/export, TUI failed draft reopen/admission/save, and the
actual registered binary command route. Parent runs the commands documented in
`docs/source-authoring.md`; source review alone is not passing evidence.
