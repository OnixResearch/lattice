## Context

`src/agent_node.rs` owns one shared model-visible prompt renderer. Command agents call it directly. `src/openai/core.rs` and the bounded Animus leaf also use it before their provider shells.

The renderer currently keeps structured inputs as typed JSON values until `append_json_input_sections` calls `serde_json::to_string_pretty`. This location is the narrow integration seam. TOON does not belong in workflow values, provider envelopes, output decoding, durable records, or stack evidence.

The reviewed upstream facts are intentionally bounded:

- The TOON website describes TOON as a JSON-data-model notation for LLM input and output.
- Its public benchmark reports strong savings against pretty JSON, but mixed data can be slightly larger than compact JSON.
- The website specification is v4.1 Working Draft, dated 2026-07-25.
- Rust `toon-format` v0.5.0 advertises specification v3.0 support.
- Rust release v0.5.0 is commit `2136cb1a35f3ed63be733bb74d36b89d3b4592dd`.

These facts do not prove a benefit for Lattice models or prompts.

## Success Contract

Completion requires an opt-in, version-labeled prompt projection with unchanged defaults and an evidence-backed comparison against compact JSON. Merely adding the crate, copying public benchmark claims, or replacing every JSON renderer is not completion.

False completion includes format fallback, current-spec claims for v3 output, TOON output decoding, default promotion, provider-wire changes, raw prompt retention, or token claims without matching target-model evidence.

## Approach Registry

| Family | Mechanism | Decision |
|---|---|---|
| Portfolio-wide replacement | Replace JSON, Nickel, or Preserves with TOON | Rejected because TOON is a model-context projection, not a canonical or interoperable authority. |
| Kamacite canonical codec | Add TOON beside Preserves as value identity | Rejected because TOON does not preserve Kamacite source or canonical identity. |
| Model-output format | Request and strictly decode TOON responses | Rejected for the pilot because provider JSON modes and existing JSON output contracts are stronger boundaries. |
| Global Lattice default | Replace pretty JSON in every prompt immediately | Rejected because it changes existing prompts before model-specific evidence exists. |
| Per-target input projection | Keep typed JSON and render one selected profile before dispatch | Selected because it isolates the experiment at the existing shared seam. |
| Compact JSON baseline | Compare TOON with compact JSON, not only pretty JSON | Selected because whitespace removal captures much of the available saving without a new notation. |

## Data and Control Flow

```text
AgentRequest.json_inputs: Vec<serde_json::Value>
        |
        v
resolve target projection policy
        |
        v
pure render_agent_prompt(request, profile)
        |
        +--> prompt text
        +--> safe projection facts
        +--> stable render error
        |
        v
command | OpenAI-compatible | bounded Animus provider shell
```

The core receives all inputs explicitly. It does not read configuration files, environment variables, clocks, provider state, or tokenizer services. The shell resolves the target profile, calls the core, enforces existing request limits, and performs the provider effect.

## Projection Policy

The human-authored configuration remains Nickel with a closed contract. Runtime Rust consumes the exported value through Serde.

The policy maps an agent target label to one closed profile:

```text
json_pretty
json_compact
toon_v3
```

A missing target entry means `json_pretty`. This preserves existing prompt bytes and headings. Unknown profile names fail configuration admission.

The map is target-scoped rather than transport-scoped. A target therefore receives the same prompt projection when its admitted route changes between command and OpenAI-compatible transport. Existing route admission still rejects ambiguous command/provider configuration.

No `auto` profile exists in this slice. Automatic shape selection would add a second policy problem before the fixed profiles have evidence.

## Pure Projection Core

The core separates value encoding from section assembly. It returns a typed result instead of using an infallible fallback.

```text
StructuredInputProjection = JsonPretty | JsonCompact | ToonV3
RenderedStructuredInput { heading, body, profile_id }
PromptRenderError { input_index, profile_id, class }
```

The implementation uses named constants for headings, profile IDs, and display-index rules. It uses explicit control flow and assertions for admitted invariants.

`JsonPretty` reproduces the existing `JSON input` heading and pretty JSON body. `JsonCompact` uses compact JSON and an exact profile label. `ToonV3` uses the pinned reviewed codec and labels the section as `TOON v3 input`.

If a selected codec cannot encode and recover an input unambiguously, the core returns an error. This includes an empty-root or edge shape if the pinned v3 profile cannot preserve it without undocumented wrapping. The shell rejects the request before command or network dispatch. It never falls back to JSON under a TOON label.

## TOON Version Boundary

The pilot names the profile `toon_v3`. It does not use an unversioned `toon` label.

The initial dependency is `toon-format` v0.5.0 with `default-features = false`. Cargo generates the lockfile update. The change must not edit `flake.lock` manually or add a flake input for a normal crates.io dependency.

Codec fixtures bind the selected crate version, profile ID, exact output, and strict round-trip result. An upstream v4 implementation requires an explicit design update and new fixtures. It must not silently change the meaning of `toon_v3`.

The runtime uses TOON for prompt input only. It does not invoke the TOON decoder on model output. Strict decoding is a fixture and evaluation tool for local projection equivalence.

## Provider and Evidence Boundary

The common rendered prompt feeds all admitted provider routes. OpenAI request envelopes remain JSON. Requested JSON output still uses provider JSON instructions and `serde_json` decoding.

Safe response metadata can include the projection profile ID, profile implementation version, and structured-input count. It must not add raw input, rendered prompt, model response, credentials, or low-entropy content hashes.

Provider usage is an observation. Lattice does not treat provider token counts as billing truth. Missing, malformed, or contradictory usage cannot support a token-saving claim.

## Pilot Corpus and Decision Rule

The checked corpus covers uniform arrays, mixed nested objects, scalar roots, empty collections, Unicode, separators, newlines, backticks, instruction-like strings, large valid JSON numbers, and heterogeneous arrays. Positive fixtures require exact value equivalence. Negative fixtures require stable rejection for malformed profile names, codec errors, ambiguous representations, malformed TOON counts, widths, or indentation, and attempted silent fallback.

A controlled shell runs the same checked tasks with `json_pretty`, `json_compact`, and `toon_v3`. It binds the exact model identifier, endpoint class, temperature, token bound, system prompt, user prompt, input order, output mode, corpus revision, and comparator revision.

The pure evaluation core compares admitted observations. TOON v3 is eligible for continued explicit use only when its total provider-reported prompt-token count is strictly lower than compact JSON for the declared cohort and no compact-JSON passing task becomes a TOON failure.

If usage is absent, token totals are not lower, a task regresses, or the model/profile identity drifts, the report is inconclusive or negative. It must block any default recommendation. A default change always requires a separate Cairn package, even after a positive pilot.

## Adversarial Review

The review covers these risks:

- misleading savings from comparison with pretty JSON only;
- v3 output mislabeled as current v4.1 TOON;
- a TOON encoding error that silently falls back to JSON;
- empty or hostile strings that make section boundaries ambiguous;
- prompt injection treated as a codec-specific security guarantee;
- different prompt bytes across command, OpenAI-compatible, and Animus routes;
- provider usage treated as exact billing evidence;
- model accuracy judged by another model instead of an exact task comparator;
- accidental changes to provider JSON, model JSON output, durable JSON, Nickel, or Preserves boundaries;
- default promotion before the declared target cohort passes.

TOON does not prevent prompt injection. The pilot preserves the same trust classification as the source structured input and makes no instruction-isolation claim.

## Risks and Trade-offs

- The added crate increases dependency and review surface even with default features disabled.
- A target-level profile can change prompt behavior for every node on that target. The explicit policy makes this authority visible.
- Compact JSON can equal or beat TOON on mixed data. The pilot can produce a negative result.
- Provider token usage can be absent or implementation-defined. Such evidence remains bounded.
- Exact v3 labeling creates migration work after Rust v4 support arrives.
- The model can understand JSON better than TOON despite fewer tokens. Exact task outcomes remain part of the gate.

## Deferred Portfolio Work

No Animus Cairn is needed yet. If Lattice validates a second consumer, a later Animus change can define a host-supplied structured-context projection contract without putting the TOON crate in a `no_std` core.

No Kamacite Cairn is needed. A future Kamacite target-artifact adapter is justified only by a real cross-language consumer. Preserves remains canonical in all cases.

## References

- [TOON format overview](https://toonformat.dev/)
- [TOON specification](https://toonformat.dev/spec)
- [TOON benchmark](https://toonformat.dev/benchmark)
- [toon-rust v0.5.0](https://github.com/toon-format/toon-rust/releases/tag/v0.5.0)
