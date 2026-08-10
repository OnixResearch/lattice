## Context

Lattice currently owns process mechanics in two places. `execute_shell` uses `tokio::process::Command`, unbounded `wait_with_output`, and a future timeout that reports failure without proving child teardown. `ProductionProcessRunner` uses `std::process::Command::output`, checks combined output only after complete buffering, and ignores the request timeout used by command-agent and command-credential callers.

`bounded-exec` revision `29dac88ecded94457572db3fdfaaaab95fa91525` provides a `no_std` validation/classification core and a standard-library shell for explicit argv, environment, working directory, bounded stdin/stdout/stderr, deadline, cancellation, process-group teardown, and bounded worker cleanup. The source is not yet published, so this design records the cutover without treating the local checkout as a durable dependency.

## Decisions

### Decision: Require publication before dependency cutover

**Choice:** Add the dependency only after the reviewed source is available from its canonical repository at one immutable exact Git revision. Align Cargo and Nix to that identity and reject sibling path dependencies.

**Rationale:** A local detached commit is not a reproducible external source. Building against `../bounded-exec`, copying its implementation, or checking in a pin that cannot resolve would manufacture completion evidence and violate the producer's adoption contract.

### Decision: Keep a pure Lattice admission core above the shared shell

**Choice:** Convert Lattice-owned process policy into a deterministic plan containing the executable request, argv, environment mode and entries, explicit working directory, stdin mode and bytes, execution limits, termination scope, and accepted-exit/truncation policy. Keep executable lookup, current-directory observation, and the `bounded_exec::run` call in a thin shell.

**Rationale:** Workflow and provider policy remains testable without process I/O. `bounded-exec` owns mechanics only; it must not absorb Lattice's authorization, parsing, redaction, cache, graph, or evidence semantics.

### Decision: Preserve route semantics while strengthening mechanics

**Choice:** Shell nodes continue to invoke the configured shell with `-c`; command agents and credential providers continue to invoke configured argv directly. Existing ambient-environment inheritance and explicit overrides remain compatible. Missing current directories are resolved into an explicit absolute request before spawn.

**Rationale:** This change is a process-safety cutover, not a command-language migration. Any future environment clearing, executable allowlist, or working-directory authority change requires a separate policy change.

### Decision: Make every execution bound explicit

**Choice:** Add typed Nickel defaults and validated runtime values for stdin bytes, stdout bytes, stderr bytes, poll interval, and teardown timeout. Preserve shell-node and command-provider deadline ownership, and add a required positive command-agent deadline instead of the current no-timeout behavior. Reject zero or unsupported bounds before spawn.

**Rationale:** `bounded-exec` intentionally has no implicit limits. Lattice must own reviewable defaults and pass a complete `ExecutionLimits` value on every request.

### Decision: Reject truncation before product parsing

**Choice:** Select an outcome policy that treats either stdout or stderr truncation as failure for all first-wave Lattice paths. Translate the bounded observation into fixed product diagnostics before shell output mapping, agent envelope decoding, or credential JSON parsing.

**Rationale:** Parsing a retained prefix as if it were complete could accept malformed or attacker-controlled output. Lattice may add a separately reviewed truncating consumer later, but no current path needs it.

### Decision: Use process-group ownership on Unix

**Choice:** Select `TerminationScope::ProcessGroup` and retain the producer's bounded non-Unix direct-child fallback claim. Tests prove direct timeout/cancellation and Unix descendant-held-pipe cleanup.

**Rationale:** Lattice runs shell commands that can create descendants. Direct-child kill does not close descendant-held pipes or stop child work after the parent exits.

### Decision: Keep asynchronous workflow orchestration outside the synchronous runner

**Choice:** Invoke the blocking bounded runner through a narrow Tokio blocking boundary for async shell-node execution. Command-agent and command-credential paths use the same adapter synchronously until their outer interfaces become async.

**Rationale:** Pipe draining, polling, and teardown stay inside one reviewed mechanism while the workflow executor does not block a Tokio worker thread. Cancellation wiring remains explicit and does not rely on future drop.

## Invariants

- Every production process request has an absolute executable and absolute working directory before `bounded_exec::run`.
- Every request carries explicit positive limits; no zero value means unbounded execution.
- Output parsing occurs only after non-truncated bounded capture is admitted.
- Timeout or cancellation never becomes a successful node/provider result.
- Raw credential values are never included in process diagnostics, safe metadata, or persisted evidence.
- `bounded-exec` observations do not become sandbox, authorization, correctness, or release-readiness claims.

## Risks / Trade-offs

- Publication is an exact external blocker; implementation cannot validate before an immutable source exists.
- Process-group termination deliberately kills descendants owned by the request and may expose workflows that relied on background survivors.
- Explicit output limits can reject previously successful high-volume commands; diagnostics and configuration must make the bound observable without exposing output bodies.
- Resolving relative configured executables through ambient `PATH` preserves current behavior but remains consumer-owned discovery, not executable authorization.
- A blocking adapter adds a Tokio blocking-task handoff for shell nodes; bounded stress tests must confirm configured workflow parallelism does not exhaust the blocking pool.
