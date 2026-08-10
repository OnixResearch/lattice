## Phase 1: Baseline and pinned profile

- [ ] [serial] Run the smallest current agent prompt, command route, OpenAI route, Animus leaf, configuration, and repository checks before core edits. Record any pre-existing failure without changing unrelated work. r[lattice.agent_runtime.structured_input_projection]
- [ ] [parallel] Review and record the exact TOON website specification, benchmark profile, Rust v0.5.0 source identity, advertised v3 compliance, license, transitive dependencies, and v3/v4 non-claim. r[lattice.agent_runtime.toon_v3_projection] r[lattice.agent_runtime.projection_evaluation]
- [ ] [serial] Add the `toon-format` v0.5.0 dependency with default features disabled, and let Cargo generate `Cargo.lock`. Do not edit `flake.lock` manually. r[lattice.agent_runtime.toon_v3_projection]
- [ ] [parallel] Add typed Nickel contract fixtures for valid target mappings and negative unknown-profile, wrong-type, and malformed-map cases. r[lattice.agent_runtime.structured_input_projection.scenario.selected] r[lattice.agent_runtime.structured_input_projection.scenario.default]

## Phase 2: Pure prompt-projection core

- [ ] [serial] Add the closed `JsonPretty`, `JsonCompact`, and `ToonV3` profile type plus target-policy resolution with exact `JsonPretty` omission compatibility. r[lattice.agent_runtime.structured_input_projection]
- [ ] [serial] Extract pure structured-input encoding and prompt-section assembly that return typed results and stable errors without I/O or fallback. r[lattice.agent_runtime.structured_input_projection] r[lattice.agent_runtime.toon_v3_projection]
- [ ] [serial] Implement the pinned TOON v3 input profile with exact versioned headings, named constants, explicit invariants, and pre-dispatch rejection for unsupported or ambiguous values. r[lattice.agent_runtime.toon_v3_projection.scenario.supported] r[lattice.agent_runtime.toon_v3_projection.scenario.rejected]
- [ ] [parallel] Add positive exact-output and strict round-trip tests for uniform arrays, mixed objects, scalar roots, empty supported collections, Unicode, separators, newlines, backticks, instruction-like strings, valid numeric edges, and heterogeneous arrays. r[lattice.agent_runtime.toon_v3_projection.scenario.supported]
- [ ] [parallel] Add negative tests for unknown profiles, codec failure, ambiguous empty roots, malformed TOON counts, widths, indentation, and every attempted silent-fallback path. r[lattice.agent_runtime.toon_v3_projection.scenario.rejected]
- [ ] [parallel] Prove omitted policy retains the current `JSON input` heading and exact pretty JSON prompt fixtures. r[lattice.agent_runtime.structured_input_projection.scenario.default]

## Phase 3: Provider shells and format boundaries

- [ ] [serial] Resolve the selected profile by admitted target and route command, OpenAI-compatible, and bounded Animus prompts through the same pure renderer before effects. r[lattice.agent_runtime.structured_input_projection.scenario.selected]
- [ ] [parallel] Add route tests that assert identical prompt projection across compatible command, OpenAI-compatible, and Animus paths, plus zero provider calls after render failure. r[lattice.agent_runtime.structured_input_projection.scenario.selected] r[lattice.agent_runtime.toon_v3_projection.scenario.rejected]
- [ ] [serial] Add bounded safe profile metadata without raw inputs, prompts, responses, credentials, or low-entropy content hashes. r[lattice.agent_runtime.projection_boundaries.scenario.evidence]
- [ ] [parallel] Add regression tests that keep OpenAI envelopes, requested JSON output, workflow JSON values, n8n interchange, durable history, replay, Cairn evidence, and existing omitted configuration unchanged. r[lattice.agent_runtime.projection_boundaries.scenario.wire] r[lattice.agent_runtime.projection_boundaries.scenario.output]

## Phase 4: Controlled pilot evidence

- [ ] [serial] Add a checked fixture corpus and pure comparison core for exact task outcomes and admitted provider prompt-token observations across `json_pretty`, `json_compact`, and `toon_v3`. r[lattice.agent_runtime.projection_evaluation]
- [ ] [parallel] Add positive comparison tests where TOON strictly beats compact JSON with unchanged passing outcomes, and negative tests for equal or higher tokens, task regression, missing usage, contradictory usage, and profile drift. r[lattice.agent_runtime.projection_evaluation.scenario.eligible] r[lattice.agent_runtime.projection_evaluation.scenario.blocked]
- [ ] [serial] Run the controlled cohort against the exact declared model profile and record the model, prompts, corpus, comparator, profile versions, token observations, outcomes, limitations, and bounded recommendation. r[lattice.agent_runtime.projection_evaluation]
- [ ] [serial] If TOON does not beat compact JSON without task regression, keep default promotion blocked and record the exact negative or inconclusive result. Do not relax the gate. r[lattice.agent_runtime.projection_evaluation.scenario.blocked]

## Phase 5: Documentation and completion

- [ ] [parallel] Update README and example Nickel configuration with explicit opt-in steps, TOON v3 labeling, compact JSON baseline, output and canonical-format boundaries, provider-usage caveats, and the upstream Rust reference. r[lattice.agent_runtime.projection_boundaries] r[lattice.agent_runtime.projection_evaluation]
- [ ] [serial] Run focused tests, `scripts/check.rs`, `octet-precommit-check`, Cairn validate and proposal/design/tasks gates, and bounded `nix flake check -L --option secret-key-files ''`. r[lattice.agent_runtime.structured_input_projection] r[lattice.agent_runtime.projection_boundaries]
- [ ] [serial] Keep `json_pretty` as the omission default during sync and archive. Create a separate Cairn change for any later default promotion or Animus extraction. r[lattice.agent_runtime.structured_input_projection.scenario.default] r[lattice.agent_runtime.projection_evaluation.scenario.eligible]
