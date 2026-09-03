# Structured-input prompt projection

Lattice keeps agent structured inputs as typed JSON values. It changes their format only when it builds model-visible prompt sections.

## Profiles

Each agent target can select one closed profile:

- `json_pretty` uses the existing `JSON input` heading and formatted JSON body.
- `json_compact` uses a `Compact JSON input` heading and compact JSON body.
- `toon_v3` uses a `TOON v3 input` heading and the pinned TOON v3 codec.

An omitted target always selects `json_pretty`. This rule preserves existing prompt bytes and behavior.

Use an explicit target entry in `agent_runtime.structured_input_projection`:

```nickel
structured_input_projection = {
  pi = "toon_v3",
  clankers = "json_compact",
}
```

The policy is target-scoped. A target keeps the same projection when its admitted route changes between command and OpenAI-compatible transport.

## Effect boundary

The pure renderer receives the typed values and selected profile. It returns rendered sections or a bounded error.

Command, OpenAI-compatible, and bounded Animus routes call this renderer before provider effects. If TOON encoding or strict recovery fails, the request stops before process or network dispatch.

The renderer never changes formats after an error. For example, TOON v3 cannot represent an empty root object with a non-empty body. Lattice rejects that input instead of placing JSON under a TOON heading.

Safe response metadata can contain these facts:

- profile ID
- implementation label
- structured-input count

It does not contain inputs, prompts, responses, credentials, or low-entropy content hashes.

## Format boundaries

Projection applies only to model-visible structured-input sections.

These formats remain unchanged:

- Nickel configuration
- typed workflow values
- OpenAI-compatible request envelopes
- requested JSON model output
- n8n interchange
- durable history and replay
- Cairn and Valence evidence
- Mantle artifacts

TOON input does not make prompt text safe from prompt injection. It also does not add provider, workflow, or effect authority.

## Pinned TOON implementation

Lattice pins a vendored `toon-format` version `0.5.0` snapshot at `src/vendor-cargo-deps/toon-format/`. The crates.io package checksum is `8f89570c1a68d73941f728cca32a4345b2ffca36667ad921af336c60309a3e7e`.

The vendor manifest removes unused CLI, TUI, example, test, and development dependencies. It also removes `serde_json/preserve_order` to preserve Lattice's existing JSON ordering and prompt bytes. No selected upstream Rust source file changes. `UPSTREAM-FILES.b3` verifies 53 selected upstream files.

The reviewed release points to commit `2136cb1a35f3ed63be733bb74d36b89d3b4592dd`. Its README advertises TOON specification v3.0 support. Its default feature enables a CLI and TUI, so Lattice disables default features.

`src/vendor-cargo-deps/toon-format/SOURCE.ncl` and `PATCH.md` record the source and manifest-only patch. `scripts/sync_toon_format.rs` reproduces the snapshot and has positive and negative source-admission tests.

Lattice compiles the five vendored library modules inside the `lattice` crate from the derived `toon-format-compile/` tree. This keeps `Cargo.lock` byte-identical: the codec adds no dependency edge, so it cannot unify `serde_json` features or replace accepted Octet identities. A local `IndexMap` alias maps the codec's ordered map to `BTreeMap`, matching crate-wide deterministic JSON ordering. The sync script derives the compile tree from the pristine record by dropping each file's trailing upstream `#[cfg(test)]` module and marking documented example fences `ignore`; those upstream tests and doctests assume `serde_json/preserve_order` key ordering and the external `toon_format` crate name, which this featureless embedding deliberately does not provide. Lattice's own round-trip corpus covers the codec on this build.

The current TOON website publishes specification v4.1 as a Working Draft dated 2026-07-25. Therefore, Lattice labels this profile `toon_v3`. It does not claim current v4.1 conformance.

Public benchmark results also show that TOON can be larger than compact JSON for mixed or deeply nested data. Lattice does not use those results as local evidence.

## Controlled pilot

The checked pilot binds these inputs:

- model: `VibeThinker-3B`
- endpoint class: controlled local OpenAI-compatible service
- profiles: `json_compact` and `toon_v3`
- exact system prompt, task corpus, runtime profile, and comparator identities
- JSON output mode and fixed runtime bounds

On 2026-09-02, the declared service at `127.0.0.1:13305` was unavailable. No task was dispatched. No model outcome or provider token count was observed.

The checked report is `inconclusive_missing_outcome`. It keeps `json_pretty` as the omission default.

A later controlled run can support continued explicit TOON use only when all observations are complete. TOON must use fewer total provider-reported prompt tokens than compact JSON. No compact-JSON passing task can fail under TOON.

Provider token counts are observations, not billing truth. Missing or contradictory counts block a positive result.

Default promotion always needs a separate Cairn change. A second real consumer and separate review are also required before any shared Animus extraction.

## Evidence files

- `policy/structured-input-projection.ncl`
- `policy/structured-input-projection-pilot.ncl`
- `tests/fixtures/structured-input-projection/pilot-profile.ncl`
- `tests/fixtures/structured-input-projection/pilot-corpus.json`
- `tests/fixtures/structured-input-projection/pilot-attempt.json`
- `tests/fixtures/structured-input-projection/pilot-observations.json`
- `tests/fixtures/structured-input-projection/pilot-report.json`

The receipts prove bounded configuration, rendering, strict round trips, and comparison decisions. They do not prove model correctness, equal accuracy, lower cost, lower latency, provider billing truth, or production readiness.
