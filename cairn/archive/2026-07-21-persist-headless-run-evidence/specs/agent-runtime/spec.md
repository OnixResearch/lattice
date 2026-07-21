# Agent Runtime Specification Delta

## ADDED Requirements

### Requirement: Inline reasoning boundary
r[lattice.agent_runtime.inline_reasoning_boundary] Lattice MUST prevent OpenAI-compatible inline reasoning markup from entering typed agent output, Animus final responses, run details, replay, or operator-visible evidence.

#### Scenario: Leading closed reasoning block yields final answer
r[lattice.agent_runtime.inline_reasoning_boundary.scenario.closed]
- GIVEN an OpenAI-compatible provider returns one leading closed `<think>...</think>` block followed by non-empty final content
- WHEN Lattice decodes the response at the provider boundary
- THEN it MUST discard the complete reasoning block and admit only the trimmed final content

#### Scenario: Plain final content remains unchanged
r[lattice.agent_runtime.inline_reasoning_boundary.scenario.plain]
- GIVEN provider content contains no reasoning marker
- WHEN Lattice decodes the response
- THEN it MUST preserve the trimmed content under the requested text or JSON output contract

#### Scenario: Ambiguous reasoning markup fails closed
r[lattice.agent_runtime.inline_reasoning_boundary.scenario.malformed]
- GIVEN provider content has an unterminated, nested, repeated, or non-leading `<think>` or `</think>` marker
- WHEN Lattice decodes the response
- THEN it MUST reject the response with a bounded diagnostic that excludes the reasoning and prompt content

#### Scenario: Cleaned JSON is parsed after reasoning removal
r[lattice.agent_runtime.inline_reasoning_boundary.scenario.json]
- GIVEN a provider returns one valid leading reasoning block followed by a JSON object
- WHEN the node requests JSON output
- THEN Lattice MUST remove the reasoning block before strict JSON decoding and return only the typed JSON value
