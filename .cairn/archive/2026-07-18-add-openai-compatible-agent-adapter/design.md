## Context

`ProductionAgentAdapter` currently plans and runs command runtimes for the `pi` and `clankers` target labels. The successful Ornith graph run replaced those commands with a temporary shell script that assembled JSON, called an OpenAI-compatible endpoint, disabled model thinking, decoded content, and enforced JSON output. That proves the endpoint but leaves production behavior coupled to ambient shell tools and bypasses the typed credential boundary.

## Decisions

### Decision: Preserve adapter-neutral graph targets

**Choice:** Keep the existing `AgentTarget` variants and add a separate `openai_compatible` runtime map keyed by the same target labels. Admission requires exactly one command or OpenAI-compatible entry for the selected target.

**Rationale:** The graph models role intent and typed outputs, not provider brands. A new node kind or target variant would couple saved workflows and TUI behavior to one protocol and require unnecessary format migration. Separate maps preserve the existing command configuration shape while making ambiguous routing explicit.

### Decision: Split deterministic protocol cores from the HTTP shell

**Choice:** Pure functions validate configuration, render the existing request prompt, construct an OpenAI chat-completions plan, and decode bounded response bytes. A thin blocking HTTP transport owns network I/O, timeout enforcement, disabled redirects, status capture, and response-size limits. The complete blocking client lifecycle runs on one named worker thread per admitted request.

**Rationale:** Planning and decoding remain testable without a server or mocks of the wider executor. The existing agent interface is synchronous and already shells out synchronously, so a bounded blocking transport preserves current orchestration semantics. Isolating client construction, execution, and destruction on the worker keeps reqwest's private runtime out of the executor's Tokio context and prevents nested-runtime shutdown panics.

### Decision: Pass secrets in an execution-only context

**Choice:** Extend the adapter call with a non-serializable `ExecutionContext` built from the executor's resolved node credential environment. OpenAI runtime config may name one environment binding for bearer authorization; plans and metadata carry only that binding name, never its value.

**Rationale:** API keys remain owned by credential providers and workflow credential references. Nickel stores no plaintext key, request/response values cannot serialize it, and deterministic planning does not depend on secret bytes.

### Decision: Fail closed on compatibility ambiguities

**Choice:** Allow only `http` and `https` endpoints without embedded credentials or fragments; require positive timeout/token/body bounds; disable redirects; accept only successful HTTP statuses and non-empty `choices[0].message.content`; never substitute `reasoning_content`; and support only text or JSON output. JSON requests declare the standard `json_object` response format, add an explicit no-fences/no-commentary JSON output contract, and still decode strictly. Optional `enable_thinking` is emitted only when configured.

**Rationale:** OpenAI-compatible servers differ in extensions and error shapes. Explicit omission is more portable than forcing extensions, while strict decoding prevents a loading page, hidden reasoning stream, truncated body, or malformed JSON from becoming workflow output.

## Risks / Trade-offs

- Configured endpoints are operator authority and may target private networks; this change validates protocol shape but does not claim SSRF isolation for trusted local config.
- Plain HTTP remains allowed for explicitly configured local endpoints such as Aspen; bearer credentials over plain HTTP are rejected so secrets require HTTPS.
- The synchronous adapter can occupy an executor worker until its bounded timeout and creates one named transport worker per admitted request. Concurrency remains bounded by executor admission, matching existing command behavior but not providing asynchronous multiplexing.
- Chat-completions text/JSON support does not claim image, artifact, tool-call, streaming, retry, session, or arbitrary OpenAI API compatibility.
