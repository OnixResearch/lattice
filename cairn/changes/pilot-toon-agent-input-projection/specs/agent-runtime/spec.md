## ADDED Requirements

### Requirement: Explicit structured-input prompt projection
r[lattice.agent_runtime.structured_input_projection] Lattice MUST retain agent structured inputs as typed JSON values until final prompt rendering and MUST select their model-visible projection through an explicit closed per-target runtime policy.

#### Scenario: Omitted policy preserves the current prompt
r[lattice.agent_runtime.structured_input_projection.scenario.default]
- GIVEN an admitted agent target has no structured-input projection entry
- WHEN Lattice renders a request with structured JSON inputs
- THEN it MUST use the existing `JSON input` heading and pretty JSON body without changing the remaining prompt sections

#### Scenario: Selected profile applies at the shared boundary
r[lattice.agent_runtime.structured_input_projection.scenario.selected]
- GIVEN an admitted agent target selects a valid `json_compact` or `toon_v3` profile
- WHEN a command, OpenAI-compatible, or bounded Animus route renders that target's request
- THEN Lattice MUST apply the selected profile only to structured-input sections and MUST give every compatible route the same rendered prompt

### Requirement: Versioned TOON v3 input projection
r[lattice.agent_runtime.toon_v3_projection] Lattice MUST implement `toon_v3` as an input-only projection through one pinned reviewed v3-compatible codec, MUST identify the profile and prompt sections as v3, and MUST reject projection errors before provider effects without switching formats.

#### Scenario: Supported input has exact value meaning
r[lattice.agent_runtime.toon_v3_projection.scenario.supported]
- GIVEN a typed JSON input has a representation that the pinned TOON v3 profile can encode and strictly recover without ambiguity
- WHEN the pure prompt renderer applies `toon_v3`
- THEN it MUST emit a non-empty version-labeled section whose validated decoded value equals the original typed JSON value

#### Scenario: Unsupported or ambiguous input fails closed
r[lattice.agent_runtime.toon_v3_projection.scenario.rejected]
- GIVEN the selected TOON v3 codec reports an error or cannot preserve an empty-root, edge, or hostile value unambiguously
- WHEN Lattice plans the agent request
- THEN it MUST return a stable bounded error before command or network dispatch and MUST NOT emit JSON under a TOON label or fall back to another profile

### Requirement: Prompt-only format boundary
r[lattice.agent_runtime.projection_boundaries] Lattice MUST confine structured-input projection to model-visible prompt text and MUST preserve existing configuration authority, provider protocols, output typing, workflow values, durable formats, and stack evidence authority.

#### Scenario: Provider wire format remains JSON
r[lattice.agent_runtime.projection_boundaries.scenario.wire]
- GIVEN a target selects `json_compact` or `toon_v3` for structured prompt inputs
- WHEN Lattice plans an OpenAI-compatible request
- THEN the provider envelope MUST remain valid JSON and MUST contain the projected text only inside the existing model-message content field

#### Scenario: Requested JSON output remains strict JSON
r[lattice.agent_runtime.projection_boundaries.scenario.output]
- GIVEN a target uses TOON v3 input projection and the agent node requests JSON output
- WHEN the provider returns content
- THEN Lattice MUST apply the existing strict JSON output contract and MUST NOT decode the response as TOON

#### Scenario: Evidence remains bounded and redaction-safe
r[lattice.agent_runtime.projection_boundaries.scenario.evidence]
- GIVEN an agent route renders or dispatches a selected structured-input projection
- WHEN Lattice records safe runtime metadata or pilot evidence
- THEN it MAY record profile, implementation-version, input-count, and bounded outcome facts but MUST NOT retain raw inputs, rendered prompts, responses, credentials, or low-entropy content hashes

### Requirement: Compact-JSON-controlled pilot evaluation
r[lattice.agent_runtime.projection_evaluation] Lattice MUST compare TOON v3 with compact JSON under the same declared model, prompt, task corpus, output comparator, and runtime bounds before recommending continued use, and SHALL require a separate change before any default promotion.

#### Scenario: Strict measured benefit permits only explicit use
r[lattice.agent_runtime.projection_evaluation.scenario.eligible]
- GIVEN a controlled cohort has complete admitted provider prompt-token observations and exact task outcomes for compact JSON and TOON v3
- WHEN TOON v3 uses strictly fewer total prompt tokens and no compact-JSON passing task becomes a TOON failure
- THEN the report MAY recommend continued explicit `toon_v3` use for that exact cohort but MUST keep omitted policy on `json_pretty` and MUST NOT claim broader model or portfolio benefit

#### Scenario: Missing benefit or evidence blocks promotion
r[lattice.agent_runtime.projection_evaluation.scenario.blocked]
- GIVEN provider usage is missing or contradictory, TOON v3 does not beat compact JSON, any passing task regresses, or the model, corpus, comparator, codec, or profile identity drifts
- WHEN the pilot evaluates the observations
- THEN it MUST report a negative or inconclusive result and MUST NOT recommend default promotion, current-spec conformance, lower billing, equal accuracy, or production readiness
