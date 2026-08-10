## Why

Lattice keeps agent structured inputs as `serde_json::Value`, but the shared prompt renderer converts every value to pretty JSON before model dispatch. This adds repeated whitespace and keys to model-visible context. It also gives operators no explicit prompt-projection policy.

TOON can reduce tokens for some structured inputs. It is not a safe global replacement. Public benchmarks show small or negative gains against compact JSON on some data shapes. The current TOON website publishes specification v4.1, while the reviewed Rust v0.5.0 implementation advertises specification v3.0.

A bounded Lattice pilot can test TOON at the final model-input boundary. The pilot must preserve typed JSON internally and retain current wire, output, configuration, receipt, and interchange formats.

## What Changes

- Add a typed per-agent-target structured-input projection policy with `json_pretty`, `json_compact`, and explicit experimental `toon_v3` profiles.
- Preserve `json_pretty` as the default for omitted policy so existing prompts remain byte-compatible.
- Extract a pure prompt-projection core that accepts typed values and an explicit profile, then returns rendered sections or a stable error.
- Route command, native OpenAI-compatible, and bounded Animus leaf prompts through the same selected projection before provider effects.
- Pin the reviewed Rust TOON implementation without default CLI or TUI features, and identify its output as `toon_v3` rather than current-spec TOON.
- Fail before model dispatch when the selected profile cannot represent an input unambiguously. Do not switch formats silently.
- Keep OpenAI request envelopes, JSON model output, workflow values, Nickel configuration, n8n interchange, durable history, and evidence formats unchanged.
- Add a checked pilot corpus and an exact outcome comparator for pretty JSON, compact JSON, and TOON v3.
- Record provider-reported prompt-token observations only when the provider supplies them. Missing or contradictory usage makes the result inconclusive.
- Require a later Cairn change before any default-profile promotion or shared Animus extraction.

## Success Criteria

- An omitted projection policy produces the current `JSON input` sections without prompt drift.
- An explicit target policy changes only that target's model-visible structured-input sections.
- Supported TOON v3 fixtures recover the same JSON value under the pinned strict decoder.
- Unsupported or ambiguous values fail before command or network dispatch without fallback.
- Positive and negative tests cover every profile, route, boundary, and format-error path.
- A controlled report compares TOON v3 with compact JSON under the same model, prompt, task corpus, and output comparator.
- The report does not recommend TOON when token evidence is absent, compact JSON is not beaten, or any previously passing task fails.
- No result claims canonical identity, current TOON specification conformance, model correctness, provider billing truth, or broad portfolio suitability.

## Non-Goals

- Replacing Nickel, Preserves, JSON APIs, n8n JSON, JSONL, CSV, Valence evidence, Cairn receipts, or Mantle artifacts.
- Requesting TOON model output or decoding provider output as TOON.
- Automatically selecting a projection from input shape.
- Promoting TOON as the default in this change.
- Adding TOON to Kamacite or making Kamacite own model-context encoding.
- Moving the policy or codec into Animus before another consumer and the Lattice evidence justify extraction.
- Claiming that token reduction proves equal model accuracy, lower cost, or lower latency.

## Impact

- **Behavior**: additive target policy; omitted policy preserves the current prompt.
- **Files**: agent runtime configuration, common prompt renderer, provider shells, Nickel examples/contracts, focused fixtures/tests, pilot evidence, Cargo dependency metadata, README references, and Cairn traceability artifacts.
- **Dependencies**: reviewed `toon-format` Rust crate with default features disabled; Cargo must generate any `Cargo.lock` change.
- **Compatibility**: provider wire JSON, output JSON, durable values, and existing configuration defaults remain unchanged.
- **Validation**: pre-change focused baselines, unit and route tests, strict codec fixtures, controlled pilot comparison, `scripts/check.rs`, Octet, Cairn gates, and bounded Nix checks.
