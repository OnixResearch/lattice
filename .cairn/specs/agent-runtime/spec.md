# Agent Runtime Specification

## Purpose

Defines the `agent-runtime` capability.

## Requirements

### Requirement: Animus leaf agent target
r[lattice.agent_runtime.animus_leaf] Lattice MUST execute the Animus runtime only as a bounded implementation of one generic agent node and MUST retain workflow graph, runner, orchestration, trigger, budget, credential, effect, persistence, and operator authority outside Animus.

#### Scenario: Admitted Animus node uses the existing runner boundary
r[lattice.agent_runtime.animus_leaf.scenario.execute]
- GIVEN a valid workflow contains a generic agent node targeting `animus` and the matching provider runtime is configured
- WHEN the Lattice runner dispatches that node
- THEN the production agent adapter MUST invoke one bounded Animus loop and MUST return its typed result as that node's output

#### Scenario: Animus cannot become an outer orchestrator
r[lattice.agent_runtime.animus_leaf.scenario.authority]
- GIVEN an Animus model response requests a graph, trigger, credential, budget, approval, persistence, or orchestration mutation
- WHEN the inner loop returns that content
- THEN Lattice MUST treat it only as ordinary agent output and MUST NOT grant or apply the requested authority outside normal Lattice admission

### Requirement: Immutable bounded Steel loop
r[lattice.agent_runtime.animus_fabric] Lattice MUST bind the first Animus target to one built-in BLAKE3-identified Steel program with an explicit two-turn ceiling and MUST reject identity drift or turn exhaustion before additional provider effects.

#### Scenario: Model response completes within the fixed turn bound
r[lattice.agent_runtime.animus_fabric.scenario.complete]
- GIVEN the checked built-in program starts without a prior model message and the model adapter returns one valid response
- WHEN the Animus loop evaluates its first and second turns
- THEN it MUST dispatch the model exactly once and return that response without evaluating a third turn

#### Scenario: Program identity drift fails closed
r[lattice.agent_runtime.animus_fabric.scenario.identity]
- GIVEN the built-in Steel source does not match its checked BLAKE3 identity
- WHEN Animus execution admission runs
- THEN Lattice MUST reject the node before invoking the model transport

#### Scenario: Turn exhaustion fails closed
r[lattice.agent_runtime.animus_fabric.scenario.exhaustion]
- GIVEN a loop would require more decisions than the Lattice turn ceiling
- WHEN it requests another decision
- THEN the bounded fabric MUST fail the node and MUST NOT perform another model or tool effect

### Requirement: Lattice-owned model and tool authority
r[lattice.agent_runtime.animus_effects] Lattice MUST execute Animus model requests only through its bounded OpenAI-compatible provider shell and MUST expose no tool definition, tool capability, tool executor, ambient credential, or fallback target in the first integration slice.

#### Scenario: Credential resolves only at provider dispatch
r[lattice.agent_runtime.animus_effects.scenario.credential]
- GIVEN the configured provider references a bearer credential binding and the agent node supplies that credential reference
- WHEN Animus requests a model dispatch
- THEN Lattice MUST resolve the credential at its OpenAI-compatible transport boundary and MUST NOT include the credential in Animus input, events, output, or metadata

#### Scenario: Tool request is denied without an effect
r[lattice.agent_runtime.animus_effects.scenario.tool-denied]
- GIVEN a Steel decision requests a tool
- WHEN the Animus runtime evaluates dispatch admission
- THEN the request MUST fail closed because no trusted definition, capability, or executor exists and MUST NOT invoke a Lattice product effect

#### Scenario: Provider failure does not fall back
r[lattice.agent_runtime.animus_effects.scenario.provider-failure]
- GIVEN the Lattice-owned model transport rejects, times out, or returns malformed output
- WHEN the Animus model adapter reports that failure
- THEN the node MUST fail with a redacted diagnostic and MUST NOT retry through Pi, Clankers, another provider, or a tool

### Requirement: Typed output and bounded evidence
r[lattice.agent_runtime.animus_output] Lattice MUST admit Animus final responses as typed text or JSON outputs and MUST expose only bounded redaction-safe runtime metadata without raw prompts, raw runtime events, model responses, or credentials.

#### Scenario: Text and JSON responses preserve output typing
r[lattice.agent_runtime.animus_output.scenario.typed]
- GIVEN an Animus loop reaches a final response compatible with the generic agent node's requested text or JSON mode
- WHEN Lattice admits the terminal report
- THEN it MUST return the matching typed graph value and safe adapter metadata

#### Scenario: Unsupported output fails before dispatch
r[lattice.agent_runtime.animus_output.scenario.unsupported]
- GIVEN an Animus target requests image or artifact output, image input, or a mutable session
- WHEN execution admission runs
- THEN Lattice MUST reject the request before Steel evaluation or provider dispatch

#### Scenario: Incomplete status is not authoritative completion
r[lattice.agent_runtime.animus_output.scenario.non-authoritative]
- GIVEN the Animus runtime stops, exhausts its turn limit, or fails without a final response
- WHEN Lattice maps the terminal report
- THEN it MUST fail the node and MUST NOT report objective or orchestration completion

### Requirement: Existing agent compatibility
r[lattice.agent_runtime.animus_compatibility] Lattice MUST preserve existing Pi, Clankers, direct OpenAI-compatible, generic-agent format, runner, cache, and orchestration behavior when the additive Animus target is not selected.

#### Scenario: Existing targets retain their routes
r[lattice.agent_runtime.animus_compatibility.scenario.routes]
- GIVEN an existing generic agent node targets Pi or Clankers with a command or direct OpenAI-compatible runtime
- WHEN production routing runs after Animus integration
- THEN Lattice MUST select the same configured route and MUST NOT invoke Animus

#### Scenario: Missing Animus configuration is node-scoped
r[lattice.agent_runtime.animus_compatibility.scenario.missing]
- GIVEN a generic agent node targets `animus` without a matching provider runtime
- WHEN production routing runs
- THEN Lattice MUST reject only that node with a clear preflight diagnostic and MUST leave workflow and orchestration state unchanged

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

### Requirement: Bounded command-agent process
r[lattice.agent_runtime.bounded_command_process] Lattice MUST execute command-backed agent adapters through the shared bounded process mechanism with an explicit positive deadline, bounded stdin, independent stdout and stderr limits, owned teardown, truncation rejection, and no fallback route.

#### Scenario: Command agent returns bounded output
r[lattice.agent_runtime.bounded_command_process.scenario.success]
- GIVEN a configured command-agent route exits before its deadline with complete output inside both stream bounds
- WHEN the production agent adapter invokes it
- THEN Lattice MUST decode the complete output under the requested typed mode and preserve bounded safe metadata

#### Scenario: Command agent timeout or truncation fails closed
r[lattice.agent_runtime.bounded_command_process.scenario.failure]
- GIVEN a command-agent route times out, is cancelled, or exceeds either stream bound
- WHEN bounded execution classifies the observation
- THEN Lattice MUST fail that node before output or handoff parsing and MUST NOT retry through another command, provider, target, or tool
