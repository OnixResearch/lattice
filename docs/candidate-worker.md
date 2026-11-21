# Candidate worker boundary

`candidate_worker` runs one bounded candidate-producing command. Generic Pi and
Clankers command agents remain unchanged; the stronger OMP route is selected
explicitly through `candidate_worker::omp::run`.

## Admission and workspace provider

`candidate_worker::Request` binds work, attempt, fence, workspace, executable,
arguments, positive deadline and bounded stdout/stderr capture. Workspace paths
are re-admitted immediately before dispatch. Malformed identifiers, escaped or
noncanonical boundaries, credentials, missing providers and unavailable roots
are refused rather than falling back from required isolation to ambient execution.

The bubblewrap route constructs a private mount/PID/user namespace, disables
nested user namespaces, drops capabilities and clears the environment. Explicit
read paths are read-only; explicit existing write paths are writable. The private
workspace root and filesystem root are remounted read-only after bind setup.
`/tmp` and `HOME=/tmp/omp-home` are private scratch space; `PATH=/usr/bin:/bin` and
the selected read-only system bases are explicit runtime grants, not host-home
access. Generic commands with `boundary.network=true` still share the host
network and cannot advertise destination-scoped confinement. The selected OMP
profile now refuses that mode and always uses a private network namespace plus
the attempt-local model broker described below.

Root `.git`/`.jj` metadata is neither a source artifact nor an admitted path.
A linked worktree pointer is not followed or exposed to the worker. Repository
commands that require metadata therefore need a separately qualified provider;
this implementation does not grant shared-ref mutation authority.

`require_isolation=false` remains a trusted local command facility. A cwd and
clear environment alone do not enforce filesystem or publication restrictions.
Do not use that facility as an isolated coding-attempt qualification.

## Explicit pinned OMP JSONL profile

The selected reference is
[`e325ffc73ab86e03bdd20a91b77d8e2b3964c906`](https://github.com/can1357/oh-my-pi/tree/e325ffc73ab86e03bdd20a91b77d8e2b3964c906).
The profile follows its [CLI reference](https://github.com/can1357/oh-my-pi/blob/e325ffc73ab86e03bdd20a91b77d8e2b3964c906/docs/cli-reference.md),
[print-mode producer](https://github.com/can1357/oh-my-pi/blob/e325ffc73ab86e03bdd20a91b77d8e2b3964c906/packages/coding-agent/src/modes/print-mode.ts)
and [session header](https://github.com/can1357/oh-my-pi/blob/e325ffc73ab86e03bdd20a91b77d8e2b3964c906/packages/coding-agent/src/session/session-entries.ts).

`candidate_worker::omp::Profile` requires the reviewed executable's BLAKE3, this exact reference
revision, the independently admitted base snapshot identity, one non-command
prompt, and a `model_broker` profile. OMP, bubblewrap, the relay and shell resolve
to immutable Nix store artifacts; their byte identities are checked before
dispatch. The model selector is exactly `lattice/MODEL`, where `MODEL` equals the
broker's wire model. The wrapper owns argv; arbitrary caller argv, resume and
interactive `/loop` are not admitted:

```text
launch --cwd /workspace --model MODEL --no-session --no-title -p --mode json
--no-extensions --no-skills --no-rules --no-context-files --no-lsp --no-pty
--tools read,edit,write --approval-mode write --max-time DEADLINEs -- PROMPT
```

The wrapper requires the workspace provider and never uses the generic
single-document agent JSON decoder. It checks bounded newline-terminated JSONL,
session version 3, session cwd, and terminal assistant/agent events. Incremental
updates and intermediate tool-use messages are not final answers. Error/aborted
terminal messages do not authorize a candidate. Missing/truncated/malformed
terminal output cannot turn exit zero into success.

`candidate_worker::omp::RunResult::raw_events()` exposes bounded raw stdout separately
from the normalized `candidate_worker::omp::Observation`. The implementation-attempt shell durably retains
both artifacts before candidate publication; raw bytes are bound independently
of normalization success and are never stored as source-file artifacts.
No worker hashes, test claims or final-response text become host candidate facts.
The result retains executable, argv and workspace-provider/environment/schema
compatibility identities. These are compatibility observations, not proof that
the reviewed executable bytes were built from the declared source revision.
Usage completeness is explicitly false: JSONL is not a complete model-request,
subagent or billing ledger, and no measured usage is invented from final text.

## Attempt-local model broker

`candidate_worker::broker::Profile` selects one canonical HTTPS endpoint (literal
loopback HTTP is also supported), one exact wire model, an optional `SecretRef`,
reviewed relay/shell paths and BLAKE3 identities, the sandbox BLAKE3, an absolute
expiry and a positive request ceiling no greater than 256. URL credentials,
queries, fragments and non-chat-completion endpoints are refused.

The host creates a private mode-0700 directory, mode-0600 Unix socket and
read-only OMP model configuration. Only that socket and public local model
configuration enter the sandbox. A pinned socat relay listens on private
loopback port 41001; the worker receives no host network namespace, provider
bearer, publisher credential or privileged host socket. Relay readiness is
checked before starting the worker. PID-namespace and existing bounded-process
teardown own the relay and worker descendants.

The broker accepts only bounded `POST /v1/chat/completions` JSON for the selected
model. It rejects other paths/models, CONNECT, ambiguous framing and oversized
input before resolving credentials. Worker headers are never forwarded;
authorization is constructed on the host through the existing credential
provider. JSON is canonically re-encoded to eliminate duplicate-key ambiguity.
Every admitted dispatch consumes the request ceiling, including an uncertain
outcome. There are no broker retries, redirects or ambient HTTP proxies.
Response bodies are bounded by the existing native HTTP limits; SSE is buffered
until the bounded upstream response completes, not streamed incrementally.
Literal credential echoes are refused rather than returned to the worker.

Mount grants are fenced to the work/attempt/fence/workspace, program/argv,
read/write roots, environment/network mode, sandbox and resource limits.
Changed bindings cannot reuse a live grant. Expiry and cancellation are checked
before credential resolution and again before dispatch. The broker stops and
joins on normal return or unwinding; an unexpected post-dispatch broker failure
cannot produce an accepted candidate. These local checks do not establish a
current external Kiln grant, compute reservation or serving-generation lease.

Standalone execution resolves only the exact explicitly selected `env` reference
on the host. Embedders supply the existing `CredentialProvider` to `omp::run`;
provider trust/currentness remain their responsibility. No secret is placed in
the request, argv, worker environment, model configuration or compatibility
receipt. Byte pins establish the selected artifacts, not their source/build
provenance or general hostile-sandbox safety.

Lattice owns this broker, request codec and compatibility projection. Kiln owns
current external work authority and independent acceptance; Aspen and Animus own
the still-unavailable compute and serving/request-admission contracts. The
repeatable checks are the bounded socket security fixtures, explicitly selected
kernel-provider scenarios and non-dispatching request-v2 CLI preparation smoke.


## Host-derived candidate and partial evidence

Host capture includes staged, unstaged and untracked ordinary file bytes: it is
not a Git-index-only diff. Only root VCS metadata is excluded; `.omp`, `.pi`,
`target` and host metadata filenames are no longer silently dropped. Symlinks,
multiply linked files, special files, non-UTF-8 paths and excessive snapshots
are rejected. Reads are bounded before allocating the full file. Byte-only
receipts reject changes to existing file modes and, on Unix, new files whose
mode is not 0644. Empty directories, xattrs and VCS state are not source artifacts.

A `CandidateReceipt` binds exact work/attempt/fence/workspace, base/candidate/diff
BLAKE3 identities and sorted changed paths including additions and deletions.
Receipt constructors and `verify_receipt_from_snapshots` use named borrowed
`candidate::Bindings`; verification independently recomputes bindings and identities.
Exit zero without a changed source path is not a candidate. Changed/added files
are retained as exact bytes; deleted files have no artifact bytes.

Stored file addresses use `store::ArtifactKey { attempt_id, relative_path }`.
This borrowed address is not a capability or an acceptance receipt. Persisted
receipt and artifact-key encodings are unchanged by the Rust API cutover.

## Runner error phases and observed termination

The selected Bounded Exec source is the vendored
`29dac88ecded94457572db3fdfaaaab95fa91525`; the following mapping depends on its
actual return paths, not on OS error text:

| Runner observation | Process adapter | Termination observation |
| --- | --- | --- |
| Invalid limits/request | `InvalidRequest` | Definitive pre-spawn rejection |
| `Io { operation: Spawn }` | `Spawn` | Spawn failed |
| `Io { operation: Wait }` | `Execution / Execution` | Observed: returned only after successful terminate-and-reap |
| Missing pipe, stream I/O/count overflow, worker disconnection/panic | `Execution / Capture` | Observed: pinned runner returns after child teardown |
| Worker teardown deadline | `Execution / Teardown` | Child termination observed, capture teardown incomplete |
| Kill/reap I/O, child teardown deadline, process identifier overflow | `Execution / Teardown` | Unknown; not proof of no-start or proof the child is still running |

Process infrastructure diagnostics use bounded safe phase labels, not raw paths,
environment values or OS error strings. The observed process API preserves
bounded stdout/stderr prefixes on timeout, cancellation and overflow. The generic
process API still rejects those outputs as before. Cancellation/timeout intent
does not manufacture an observed stop; runner teardown errors remain typed
post-start failures instead of becoming `Spawn`.

`candidate_worker::Outcome::Interrupted` keeps the typed phase/termination,
available bounded stdout/stderr and a best-effort workspace snapshot. Its durable
result is always `Unknown`, never a usable candidate receipt. Observed artifact
identities and exact file bytes are retained by `candidate_worker::run_and_publish`.
A snapshot while termination is
unknown is explicitly non-atomic. No path here releases the workspace, admits a
successor, recovers or retries the attempt. Durable `Unknown` always reconciles
to `Incomplete`, even when partial artifacts exist. Successful child cleanup and
incomplete capture cleanup are distinct from unknown child termination.

The pinned producer does not return partial stdout/stderr in `RunError`; the
adapter cannot truthfully reconstruct those discarded bytes. Existing workspace
files are retained, and missing capture stays unavailable. Controller loss or
publication failure is not proof of external effect failure.

## Qualification and non-claims

The local child-marker regression exercises the real bounded process route,
then injects post-start capture/teardown observations. It checks retained file
and available event bytes, their identities, durable Unknown,
observed-versus-unknown termination and read-only reconciliation. It does not
reproduce a kernel kill/reap error or qualify OMP.

Live provider/OMP tests are explicitly ignored unless selected. They require
reviewed binaries, enabled kernel namespaces and an authorized model endpoint:

Executable byte pins use 64 lowercase hexadecimal BLAKE3 digits, as emitted by
`b3sum`, without a `b3:` prefix. The selected worker validates this format before
opening its broker or dispatching the process.

```sh
cargo test --locked --lib process::shell::failure_tests
cargo test --locked --lib candidate_worker
cargo test --locked --test test_store candidate_attempt
cargo test --locked --test test_local_process
# With LATTICE_BWRAP and LATTICE_SOCAT set to reviewed immutable executables:
cargo test --locked --lib bubblewrap_ -- --ignored
# OMP additionally needs reviewed source/build and byte pins:
# LATTICE_OMP, LATTICE_OMP_REVISION, LATTICE_OMP_BLAKE3,
# LATTICE_BWRAP_BLAKE3, LATTICE_SOCAT_BLAKE3, LATTICE_WORKER_SHELL,
# LATTICE_WORKER_SHELL_BLAKE3, LATTICE_OMP_BASE_URL, LATTICE_OMP_MODEL,
# and optionally LATTICE_OMP_CREDENTIAL_ENV naming the host-only env reference:
cargo test --locked --lib candidate_worker::tests::isolation::omp_worker_produces_a_sandboxed_candidate -- --ignored --exact
# The same executable/model pins, but no live endpoint or credential, cover
# actual OMP failure/cancellation against bounded local fault servers:
cargo test --locked --lib candidate_worker::tests::isolation::faults:: -- --ignored --nocapture
```

The retained [source-qualification record](../.cairn/changes/preserve-candidate-worker-observations/evidence/source-qualification.json)
records the reviewed OMP 18.18.7 build, its executable bytes, the real
`underclass/gpt-6.1-sol` code candidate and the selected-route failure/cancellation
fixtures. Provider credentials stayed in the host OMP auth gateway; the isolated
worker received only its bounded exact-model broker route. The normal run used
three logical broker calls; these are not complete provider-attempt or billing
telemetry. Four separate actual namespace fixtures cover the provider boundary.
These observations grant no independent acceptance, publication, merge,
external exactly-once or production-readiness claim.

### Durable continuation remains unsupported

The pinned producer's
[`restoreInterruptedTurn`](https://github.com/can1357/oh-my-pi/blob/e325ffc73ab86e03bdd20a91b77d8e2b3964c906/packages/coding-agent/src/session/agent-session.ts#L13390-L13419)
marks unfinished tool calls outcome-unknown and invokes retry. Its
[`retry` path](https://github.com/can1357/oh-my-pi/blob/e325ffc73ab86e03bdd20a91b77d8e2b3964c906/packages/coding-agent/src/session/turn-recovery.ts#L3445-L3506)
can execute that tool batch again with a fresh retry budget. This is not
no-effect restoration of host-accepted results.

Issues #7/#12 still need a producer-owned bounded headless continuation contract:
stable model/tool invocation identities, accepted result/artifact references and
cursor commits, no-effect reconstruction, and a barrier for uncertain calls.
The consumer must revalidate current attempt/fence, workspace, runtime and service
authority. Session IDs, transcripts and RPC pagination cursors cannot substitute.
The single-shot `--no-session` profile stays enforced; Unknown reconciliation
retains evidence without redispatch, and Animus tools remain disabled.

## Coordinator-admitted implementation consumer

The separately selected `lattice implementation-attempt` commands consume the
existing immutable envelope and isolated OMP worker. They do not change the
guarded drain/review routes or acquire Kiln's global work authority:

```sh
lattice implementation-attempt prepare --request seed.json --output request.json
lattice implementation-attempt run --request request.json --state /var/lib/lattice/implementation
lattice implementation-attempt recover --request request.json --state /var/lib/lattice/implementation
lattice implementation-attempt cancel --request request.json --state /var/lib/lattice/implementation
lattice implementation-attempt inspect --request request.json --state /var/lib/lattice/implementation
lattice implementation-attempt export-candidate --request request.json --state /var/lib/lattice/implementation --output-parent /var/lib/lattice/candidates
```

The strict `lattice.implementation-attempt-run.v3` fields and canonical profile
codec are in `src/implementation_attempt/cli.rs`. Preparation reads the actual
workspace snapshot and emits snapshot/profile/envelope/request-byte identities;
it reports `admitted:false` and grants no authority. The installed operator
request binds current work/version/attempt/fence/source/spec/base and an expiry.
Only the trusted Kiln dispatcher may admit it through current claim/resource
grants. Publisher authority remains unavailable; selected model credentials stay
in the host broker.

Version 3 adds two required fields. `worker_compute` is the worker's compute
requirement (`host_local`, or `provider_execution` with an optional
direct-accelerator requirement); it is part of `worker_profile_identity`, so a
changed requirement changes the profile revision. `execution_binding`
(`lattice.execution-binding.v1`) declares the expected compute (`host_local` or
one `provider_allocation` with allocation, host, consumer and owner generation)
and the model routes. The OMP worker has exactly one route named `primary`,
whose identity is derived from `omp.model_broker`; preparation normalizes that
single route's identity. No-model, extra-route, substituted-route and
mismatched-placement declarations are refused before any effect. Version-2
requests are refused before any effect with a diagnostic naming explicit
readmission as version 3, as version-1 requests required readmission as
version 2. The Kiln dispatcher must readmit requests as v3. When it reserves a
provider allocation, it must use Lattice's envelope `attempt_id` as the
producer `attempt_id`, use globally unique operation ids, and declare the
expected producer owner generation; that generation is not Lattice's `fence`.

A private one-shot execution intent is fsynced before the worker starts. The
admitted execution binding is fsynced once to `execution-binding.json` before
that intent. The standalone CLI's compute and serving-owner ports are always
unavailable, so a provider-allocation request stops before the binding, intent
or worker, and a request on a served model route is refused before upstream
contact. An embedding host supplies read-only ports through
`run_command_with_ports`. After the intent and immediately before spawn,
a provider binding is re-observed and revalidated; a refusal is written once to
`binding-revalidation.json`, the worker never starts, and recovery reports
`not_dispatched_binding_rejected`. The `Scope::Attempt` usage observation with
frozen binding provenance and per-request serving coverage is stored before the
attempt result. See [worker compute binding](worker-compute-binding.md) and
[worker serving binding](worker-serving-binding.md).

An optional `worker_capability` (`lattice.worker-capability.v1`) names the
agent definition and declares bounded task families and tools. It is framed
into `worker_profile_identity` only when present. A malformed declaration is
refused before any effect. `inspect` prints `lattice.worker-inventory.v3`,
with the agent revision lifecycle's slots when `--revision-store` is given;
see [worker inventory](worker-inventory.md).

An optional `workflow_agent` names a workflow-agent revision (Lattice #25) as
`{"definition": "blake3:<64 hex>", "revision": "<64 hex>"}`. It is
per-attempt, bound by the exact request bytes and not framed into
`worker_profile_identity`; absent, the request keeps its bytes, request
identity and profile revision. A malformed pair is refused before any effect.
When present, the attempt needs `--revision-store`, and its session is
admitted, permitted, closed and its export checked in the workflow-agent
definition's lifecycle instead of the worker's, only when the stored bundle
binds this request's exact profile revision. An unknown, mismatched or
revoked pair, or a plain agent bundle named as one, is
`revision_not_admitted` with `dispatched:false`, before any fence record,
binding or intent; see [attempt seam](agent-revisions.md#attempt-seam).

### Profile revision framing (breaking)

`worker_profile_identity` is the `lattice-worker-profile-revision-v2` content
identity, spelled as bare lowercase hex like every earlier revision. It frames
only profile-level configuration and the reviewed harness template: the
request schema, the OMP launch prefix `launch --cwd /workspace` and the OMP
print options in argv order (read from the constants the shell passes to the
worker, so changing any reviewed flag such as the exposed tools, the output
mode, `--no-session` or the approval mode is a new revision), the program and
sandbox program, the OMP source revision, executable BLAKE3 and model, the
broker's schema, endpoint, model, credential reference, relay and shell
artifacts, sandbox BLAKE3 and request ceiling, the read and write paths,
`output_limit_bytes`, `allow_unchanged`, `worker_compute` and, when declared,
the canonical capability. Attempts of one profile therefore share one
revision. A new executable, model, route, reviewed harness flag, tool set,
compute requirement or capability is a new revision of the same definition.
See [what the revision frames](agent-revisions.md#what-the-revision-frames).

Per-attempt values are not part of the revision. The base snapshot, task
prompt, broker expiry and workspace root are bound by the exact request bytes.
Their BLAKE3, the request identity, is the attempt usage observation's
`dispatch_id` and the digest Kiln seals to its dispatch plan. Without
`--revision-store` it is also the whole one-shot `intent.json` content, as
before. With `--revision-store` (see
[agent revisions](agent-revisions.md#integration)), `intent.json` is instead
one canonical `lattice.implementation-attempt-intent.v1` record carrying
`request_identity`, `session` (the envelope identity), `admission`
(`managed` or `unmanaged`) and `pin` (the managed session's pin, else `null`),
even when the definition is unmanaged. An intent without an admission field
reads as `unmanaged`. The candidate receipt must also match the request's base
snapshot. Work, attempt, fence, workspace, base commit, source and policy stay
in the envelope and its `admitted_envelope` identity, which the execution
binding frames.

This changes every profile revision. A request whose envelope names a revision
of the earlier framing is refused before any effect with a typed
`profile_revision_superseded` refusal. That includes every attempt retained
before this change: `run` and `recover` never re-execute or recover it, and its
state is left untouched. Retained attempts must be reconciled by their own
coordinator, and new work must be prepared again and dispatched as a new
attempt.

### Coordinator transport

Kiln's `kiln-lattice-dispatch --worker-request` runs `run` and, after a
possible earlier dispatch, `recover` with a frozen copy of the exact request
bytes. Its per-attempt directory key is the same
`state/<blake3(state/<attempt>-<fence>)>` as Lattice's, and it reads
`artifacts.omp_observation` from there. The request may carry two optional
coordinator fields, neither of which is part of the profile revision:

- `network` states the coordinator's workspace network. Only `false` is
  admitted: the OMP worker never receives a network device, only its brokered
  model route. `true` is refused before any effect.
- `dispatch_plan` is the coordinator's dispatch-plan identity (64 lowercase
  hex). The exact request bytes bind it, and every observation echoes it.

Before the binding and intent, a run records its attempt at
`works/<blake3(work_id)>/<fence>`. A request whose fence is lower than one
already recorded for its work, or that names a different attempt at a
recorded fence, is refused. A request whose bytes differ from an existing
intent is refused as substituted. Each refusal prints one
`lattice.implementation-attempt-refusal.v1` record and the command exits
non-zero:

```json
{"schema":"lattice.implementation-attempt-refusal.v1","reason":"stale_fence","work_id":"...","attempt_id":"...","fence":7,"dispatched":false}
```

`dispatched` says whether a worker effect of the refused attempt may have
started, not whether this request dispatched anything. It is `true` whenever
the attempt directory holds an execution intent other than recovery's closure
marker, so a coordinator must never settle such an attempt as refused before
any effect or run it again.

Fencing is atomic per work item within one state root. Each work's fence
records live in the directory `works/<blake3(work_id)>` inside the state root,
and every process that writes there, or writes an intent of that work, first
takes an exclusive `flock(2)` lock on that directory (`std::fs::File::lock`).
Waiters block. The lock never expires, and the kernel releases it when the
holder closes it or exits. While holding it, a run:

1. checks its fence and records its attempt;
2. persists its execution binding and candidate store;
3. rechecks that the record at its fence still names its attempt and that no
   higher fence is recorded;
4. creates its execution intent.

It releases the lock before any worker effect. A repair step takes the same
lock to recheck the fence before it writes `repair/step-N.intent`. Recovery
takes it to check for a crossed attempt and to create its closure marker. A
`stale_fence` or `crossed_attempt` refusal reads `dispatched` from the refused
attempt's directory under the lock. So in one state root, a `stale_fence`
refusal with `dispatched:false` means that attempt has no execution intent
and no run of it can create one there. A run that had passed its check before
the higher fence was recorded is refused at step 3 and starts no worker. Two
state roots share no lock and no records, so this guarantee does not span
them; Kiln remains the fence authority across them.

| `reason` | `dispatched` | When |
|---|---|---|
| `stale_fence` | `true` when the refused attempt's directory holds an execution intent, else `false`; read under the fence lock | A higher fence of the work is already recorded |
| `crossed_attempt` | Same as `stale_fence` | Another attempt is recorded at this work and fence |
| `substituted_request` | `true` | The attempt directory holds the intent of other request bytes, which may have executed; this request dispatched nothing |
| `profile_revision_superseded` | `true` when the attempt directory holds an intent other than the closure marker, else `false` | The envelope's revision is not the current framing; the attempt is never executed or recovered |
| `revision_not_admitted` | `false` | With `--revision-store`, the managed definition admits no new session of this revision, or the named `workflow_agent` is unknown, binds another profile revision or is a plain agent bundle; nothing is written ([agent revisions](agent-revisions.md#integration)) |
| `revision_blocked` | `false` | With `--revision-store`, the managed session was blocked right before spawn; the worker never started |
| `export_revoked` | `true` | `export-candidate` of a managed attempt that executed, after its session, pinned revision or qualification was revoked; nothing is exported |
| `lineage_blocked` | `false` | The request's Kiln budget lineage reports a blocker or serving backpressure; nothing is written ([repair and lineage](#bounded-same-attempt-repair-and-budget-lineage)) |
| `lineage_exhausted` | `false` | The request's Kiln budget lineage cannot cover the first step; nothing is written |
| `repair_ports_unavailable` | `false` | The request has a repair policy but the embedder supplied no worker and check ports, as the standalone CLI never does; nothing is written |

The three agent-revision reasons also carry `revision_refusal`, the
lifecycle's own refusal (its `code` is, for example, `session_revoked`), and
the two lineage reasons carry `lineage_refusal`. Kiln remains the
fence authority; these records only stop one state root from executing a
superseded or crossed request.

Re-entry with the exact request only reconciles retained results and reports the
persisted binding identity; it never calls the compute port, rebinds or
releases compute. A duplicate `run` of a request that already has an intent
reconciles exactly as `recover` does and prints the retained observation.
Changed requests and missing/incomplete evidence never authorize another
execution. When `recover` finds no intent, the worker cannot have started,
because the intent precedes every worker effect. Recovery then closes the
attempt by exclusively creating its intent path with a
`closed-before-execution:` marker and prints
`status:"not_started_closed_by_recovery"` with `result:null`. A racing or later
`run` of that attempt can no longer create its intent, so it never dispatches.
Cancelling a closed attempt reports `not_started:true`, and `inspect` reports
that the attempt has no worker instance. Every observation, including
`not_dispatched_binding_rejected`, carries the exact `envelope` and
`request_identity` that Kiln checks.

A worker blocked before it emitted any event, for example because its broker
socket could not be bound, retains the blocker as its unavailable OMP
observation next to empty raw events. Recovery accepts that pair and reports the
blocked result; it never invents a terminal. The broker socket lives at
`<workspace parent>/.lattice-model-<32 hex>/model.sock`, so the workspace parent
must leave room within the 108-byte Unix socket path limit.

Raw JSONL and normalization artifacts are separately fsynced before the
candidate result. Recovery bounds their reads, rejects symbolic links, checks
the raw digest against the owned stdout observation, and independently repeats
normalization. Missing, substituted, or contradictory artifacts cannot become a
published result. Version 2 removed the old `network_namespace` field rather
than accepting shared-network compatibility requests. Broker expiry cannot
exceed the enclosing attempt expiry. This shell creates no external
authorization grant.

Cancel writes only an exact persisted request; the running owner observes it and
uses the existing process-group cancellation. Requested cancellation is not a
termination receipt. Controller loss or unknown teardown retains the workspace
and usage uncertainty rather than releasing it for a successor.

A `cairn_artifact` source names its exact native stage context as
`change/stage/artifact` and uses its own identity tag. It is never a relabeled
coding task. Issue and task identity bytes are unchanged; the issue source wire
name is `github_issue`, which is the name Kiln sends.

An exit-zero unchanged workspace is a typed `Unchanged` observation, still a
generic failed/no-candidate result. Only explicit `allow_unchanged:true` plus
complete OMP terminal JSONL retains a separate
`lattice.unchanged-candidate-observation.v1` proposal. Its host-derived base and
candidate identities must match and its native diff/path set is empty. Generic
Produced receipts still reject empty changes. Kiln must independently validate
both already-satisfied policies and execute actual acceptance before terminal
completion; no empty PR or remote landing is inferred.

`export-candidate` reads only a complete retained result. It rechecks the frozen
request, native receipt or explicitly admitted unchanged observation, workspace
identity, and retained changed-file bytes before exporting. The output parent
must already exist, be canonical and absolute, be owned by the dispatcher user
and group, and not overlap the workspace, state, or request. The command creates
`ATTEMPT-FENCE/` with mode `0750`; files use `0640` plus source executable bits.
The native evidence files are reserved and cannot be supplied by the workspace.
Exact replay verifies every expected directory and file, content, ownership,
link count, and mode without recursively traversing unexpected subtrees. Missing,
additional, substituted, symbolic-link, or hard-linked members fail closed.
The summary always reports `accepted:false` and `integrated:false`; exporting
never starts a worker or grants publication authority.

### Bounded same-attempt repair and budget lineage

Two optional request fields extend the v3 request. Like `worker_capability`,
`network` and `dispatch_plan`, an absent field is not serialized, so a request
without them keeps its bytes, request identity and profile revision and runs
exactly as before.

`repair` (`lattice.implementation-attempt-repair-policy.v1`) is the admitted
repair policy:

```json
{"schema":"lattice.implementation-attempt-repair-policy.v1",
 "failure_classes":["test"],"max_iterations":2,"max_elapsed_ms":60000,
 "max_model_requests":3,"max_tool_invocations":6,"step_tool_invocations":1,
 "check_executor":"<64 hex>",
 "checks":[{"name":"unit","configuration":"<64 hex>"}]}
```

Failure classes and checks are strictly sorted, at most 16 each; class and
check names are lowercase ASCII letters, digits, `_` and `-`, at most 64
bytes. The reserved class `no_candidate` admits repairing a step that
produced no new candidate. `max_iterations` (1 to 8) counts repairs after the
first step, and every step is one envelope turn, so `max_iterations + 1` must
not exceed `budget.max_turns`. `max_elapsed_ms` must fit `budget.max_seconds`.
One step reserves the broker's `max_requests` model requests and
`step_tool_invocations` plus one tool invocation per declared check; the
aggregate budgets must cover one such step. A policy cannot be combined with
`allow_unchanged` or served model routes. The profile revision of a request
with a policy is a domain-separated BLAKE3 of its single-shot revision and the
policy identity, so a changed policy is a new revision of the same definition.

The standalone CLI has no ports for repair and refuses such a request as
`repair_ports_unavailable`. An embedder passes `Ports::repair` to
`run_command_with_ports` with a `WorkerPort` (one bounded step in the attempt
workspace), a `CheckPort` (one declared check of one candidate revision) and
a clock; the loop runs synchronously on the calling task. After the usual
fence, binding, intent, revalidation and revision permit, the shell:

1. re-observes the workspace and refuses it unless it equals the admitted
   `base_identity`;
2. asks the pure loop before every spawn, which stops on cancellation, an
   unreadable clock, the elapsed bound, the iteration bound, the aggregate
   budgets and the lineage, in that order;
3. under the fence lock, rechecks that the attempt is still recorded at its
   fence and that no higher fence is recorded, then writes
   `repair/step-N.intent` once before the worker effect. If the check fails,
   that step never spawns and the loop ends as `fence_superseded`. This is a
   post-dispatch outcome, retained and printed like any other disposition,
   because the attempt's intent exists and earlier steps may have had effects.
   It is never a refusal and never reports `dispatched:false`;
4. captures the workspace after an exited step. A tree equal to the parent or
   the base is no candidate; otherwise a native receipt against the admitted
   base and a candidate revision binding envelope, attempt, fence, iteration
   and parent are derived on the host. The worker's exit status, files and
   claims are never check results;
5. invokes every declared check against that exact revision and classifies the
   observations: a skipped, missing, duplicated, undeclared, reconfigured,
   crossed or self-approved (run by the worker profile itself) check, or one
   from any executor other than `check_executor`, rejects the evidence;
6. writes `repair/step-N.json` once and repeats only for failures inside the
   admitted classes.

The disposition is one of `correct_candidate`, `repaired_candidate` (with
`repairs`), `budget_exhausted` (with its `bound`: `iterations`, `elapsed`,
`model_requests`, `tool_invocations` or `lineage` and its dimension),
`cancelled`, `uncertain` (`worker_outcome`, `check_outcome` or `clock`),
`not_repairable` (with the classes), `checks_rejected` (with the defect),
`lineage_blocked`, `kiln_owned` or `fence_superseded`. A worker that proposes
`changed_work` or a `new_semantic_attempt` ends the loop as `kiln_owned`; a
step whose binding names other work, version or task, or another attempt,
fence or envelope, is refused before it is recorded. Repair steps receive the
parent revision and the admitted failure classes only, never a transcript or
check output. The
`lattice.implementation-attempt-repair-observation.v1` record (envelope,
request and worker-request identities, execution binding, policy and lineage
identities, disposition, revisions with their receipts, usage, draws and
unaccounted dimensions, `accepted:false`, `integrated:false` and the
dispatch plan when present) is written once to `repair/disposition.json` after
the attempt's `Scope::Attempt` usage observation, and printed. Recovery and a
duplicate `run` print the retained disposition; without one the outcome is
unknown and nothing runs again. No disposition is acceptance: Kiln #46 keeps
independent verification. `export-candidate` and `inspect` do not yet read
repair records, so a repaired candidate is not exported and its instance is not
projected from its disposition.

Usage is attributed per phase (`preparation`, `edit`, `check`, `repair`) with
the existing `UsageProvenance`. A worker step is charged the larger of its
reservation and its reported count per dimension, or the reservation when the
count is unknown (`provenance:"unknown"`, counts `null`). Checks are host
executions with a known count and no model requests. Failed, cancelled and
uncertain steps are charged like any other.

`lineage` (`lattice.kiln-budget-lineage.v1`) maps Kiln #61's admitted
recursive budget lineage at Kiln
[`339116f68e762e4caf0d05a9ac73a6e97a74b67d`](https://github.com/OnixResearch/kiln/tree/339116f68e762e4caf0d05a9ac73a6e97a74b67d/crates/kiln-core/src/fair_admission).
Kiln code is not vendored:

| Lattice field | Kiln source |
|---|---|
| `root` | `LineageExplanation.root` |
| `account` | `LineageExplanation.account` |
| `parent` | the account record's `parent` (`null` exactly for the root) |
| `depth` | `LineageExplanation.depth`, at most `MAX_LINEAGE_DEPTH` (8) |
| `remaining` | the account's `available` `Amounts`, at most `MAX_DIMENSIONS` (16) |
| `blocker` | `LineageExplanation.blocker`: `denied`, `expired`, `cancelled` or `budget_exhausted` |
| `backpressure` | `ServingExplanation.gate`: `eligible`, `waiting`, `overloaded`, `stale`, `incomplete` or `incompatible`, or `none` without a serving owner |

Identities are 64 lowercase hex digits. Lattice charges the dimensions
`model_requests` and `tool_invocations`; a Kiln dispatcher maps its accounting
aliases to these names. Before any write, and before every later step, a
blocker, a gate other than `none` or `eligible`, a zero remainder in any
dimension, or a remainder below the next step's reservation in a charged
dimension refuses the spawn. Every step and check of the attempt draws from
this one account; nothing is minted, refunded or reset. A child account is
admitted only as this account's direct child in the same root with a ceiling
within the remainder in every one of its dimensions. Other dimensions are
never drawn and are listed in `unaccounted_dimensions`: their usage stays
unknown to Kiln. A single-shot attempt with a lineage reports
`budget_lineage` in its observation with a conservative draw of one broker
ceiling and `tool_invocations` unaccounted. The lineage is a frozen snapshot:
Lattice does not observe Kiln's current ledger.

Qualification is fixture-level: the tests in
`src/implementation_attempt/cli/tests/repair.rs` drive the real `run` and
`recover` shell with a fixture worker, check port and clock. No OMP process,
check executor, model endpoint or Kiln lineage has been exercised, and the
pinned OMP route remains single-shot.


The scoped broker extends the previously published Lattice worker route at
`06e551e5b08ce78a46d03af5b5870ef1de81ac00`. Consumers must pin the actual source
archive/executable identities. Kiln #46 still owns independent verification and
canonical CI acceptance; its current contract admits supplied observations but
does not execute the verification profile. A live OMP/coordinator pilot and
external current-grant/worker/compute/service qualification remain separate.
