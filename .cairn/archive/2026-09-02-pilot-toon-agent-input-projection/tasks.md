## Phase 1: Baseline and pinned profile

- [x] [serial] Run the smallest current agent prompt, command route, OpenAI route, Animus leaf, configuration, and repository checks before core edits. Record any pre-existing failure without changing unrelated work. r[lattice.agent_runtime.structured_input_projection]
- [x] [parallel] Review and record the exact TOON website specification, benchmark profile, Rust v0.5.0 source identity, advertised v3 compliance, license, transitive dependencies, and v3/v4 non-claim. r[lattice.agent_runtime.toon_v3_projection] r[lattice.agent_runtime.projection_evaluation]
- [x] [serial] Integrate the exact reviewed `toon-format` v0.5.0 library source through a reproducible vendored boundary without changing the Cargo dependency graph or editing `flake.lock`. r[lattice.agent_runtime.toon_v3_projection]
- [x] [parallel] Add typed Nickel contract fixtures for valid target mappings and negative unknown-profile, wrong-type, and malformed-map cases. r[lattice.agent_runtime.structured_input_projection.scenario.selected] r[lattice.agent_runtime.structured_input_projection.scenario.default]

## Phase 2: Pure prompt-projection core

- [x] [serial] Add the closed `JsonPretty`, `JsonCompact`, and `ToonV3` profile type plus target-policy resolution with exact `JsonPretty` omission compatibility. r[lattice.agent_runtime.structured_input_projection]
- [x] [serial] Extract pure structured-input encoding and prompt-section assembly that return typed results and stable errors without I/O or fallback. r[lattice.agent_runtime.structured_input_projection] r[lattice.agent_runtime.toon_v3_projection]
- [x] [serial] Implement the pinned TOON v3 input profile with exact versioned headings, named constants, explicit invariants, and pre-dispatch rejection for unsupported or ambiguous values. r[lattice.agent_runtime.toon_v3_projection.scenario.supported] r[lattice.agent_runtime.toon_v3_projection.scenario.rejected]
- [x] [parallel] Add positive exact-output and strict round-trip tests for uniform arrays, mixed objects, scalar roots, empty supported collections, Unicode, separators, newlines, backticks, instruction-like strings, valid numeric edges, and heterogeneous arrays. r[lattice.agent_runtime.toon_v3_projection.scenario.supported]
- [x] [parallel] Add negative tests for unknown profiles, codec failure, ambiguous empty roots, malformed TOON counts, widths, indentation, and every attempted silent-fallback path. r[lattice.agent_runtime.toon_v3_projection.scenario.rejected]
- [x] [parallel] Prove omitted policy retains the current `JSON input` heading and exact pretty JSON prompt fixtures. r[lattice.agent_runtime.structured_input_projection.scenario.default]

## Phase 3: Provider shells and format boundaries

- [x] [serial] Resolve the selected profile by admitted target and route command, OpenAI-compatible, and bounded Animus prompts through the same pure renderer before effects. r[lattice.agent_runtime.structured_input_projection.scenario.selected]
- [x] [parallel] Add route tests that assert identical prompt projection across compatible command, OpenAI-compatible, and Animus paths, plus zero provider calls after render failure. r[lattice.agent_runtime.structured_input_projection.scenario.selected] r[lattice.agent_runtime.toon_v3_projection.scenario.rejected]
- [x] [serial] Add bounded safe profile metadata without raw inputs, prompts, responses, credentials, or low-entropy content hashes. r[lattice.agent_runtime.projection_boundaries.scenario.evidence]
- [x] [parallel] Add regression tests that keep OpenAI envelopes, requested JSON output, workflow JSON values, n8n interchange, durable history, replay, Cairn evidence, and existing omitted configuration unchanged. r[lattice.agent_runtime.projection_boundaries.scenario.wire] r[lattice.agent_runtime.projection_boundaries.scenario.output]

## Phase 4: Controlled pilot evidence

- [x] [serial] Add a checked fixture corpus and pure comparison core for exact task outcomes and admitted provider prompt-token observations across `json_pretty`, `json_compact`, and `toon_v3`. r[lattice.agent_runtime.projection_evaluation]
- [x] [parallel] Add positive comparison tests where TOON strictly beats compact JSON with unchanged passing outcomes, and negative tests for equal or higher tokens, task regression, missing usage, contradictory usage, and profile drift. r[lattice.agent_runtime.projection_evaluation.scenario.eligible] r[lattice.agent_runtime.projection_evaluation.scenario.blocked]
- [x] [serial] Run the controlled cohort against the exact declared model profile and record the model, prompts, corpus, comparator, profile versions, token observations, outcomes, limitations, and bounded recommendation. r[lattice.agent_runtime.projection_evaluation]
- [x] [serial] If TOON does not beat compact JSON without task regression, keep default promotion blocked and record the exact negative or inconclusive result. Do not relax the gate. r[lattice.agent_runtime.projection_evaluation.scenario.blocked]

## Phase 5: Documentation and completion

- [x] [parallel] Update README and example Nickel configuration with explicit opt-in steps, TOON v3 labeling, compact JSON baseline, output and canonical-format boundaries, provider-usage caveats, and the upstream Rust reference. r[lattice.agent_runtime.projection_boundaries] r[lattice.agent_runtime.projection_evaluation]
- [x] [serial] Run focused tests, `scripts/check.rs`, `octet-precommit-check`, Cairn validate and proposal/design/tasks gates, and bounded `nix flake check -L --option secret-key-files ''`. r[lattice.agent_runtime.structured_input_projection] r[lattice.agent_runtime.projection_boundaries]
- [x] [serial] Keep `json_pretty` as the omission default during sync and archive. Create a separate Cairn change for any later default promotion or Animus extraction. r[lattice.agent_runtime.structured_input_projection.scenario.default] r[lattice.agent_runtime.projection_evaluation.scenario.eligible]
