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

### Optional transient systemd unit launcher

By default, `candidate_worker::run` and `candidate_worker::omp::run` spawn
`bwrap` as a direct child through `candidate_worker::sandbox::launcher::Direct`,
which works on hosts without systemd.

When a dispatcher runs in a hardened systemd unit (`ProtectKernelTunables=yes`,
`RestrictAddressFamilies=AF_UNIX`), direct-child `bwrap` fails because
`--unshare-net` needs `AF_NETLINK` to bring up private loopback and
`--proc /proc` needs writable procfs mount setup. Callers can instead pass
`candidate_worker::sandbox::launcher::Systemd` to
`candidate_worker::run_with_launcher`,
`candidate_worker::run_and_publish_with_launcher`, or
`candidate_worker::omp::run_with_launcher`.

`Systemd` launches the byte-identical `bwrap` command inside a transient systemd
service unit (`--wait --pipe --collect --quiet --service-type=exec`) carrying
only the properties `bwrap` needs (`ProtectKernelTunables=no`,
`RestrictAddressFamilies=AF_UNIX AF_NETLINK`, `KillMode=control-group`,
`KillSignal=SIGKILL`, `TimeoutStopSec`, and a crash-safety `RuntimeMaxSec`
backstop). On cancellation, deadline, or output-limit stops, a scoped watchdog
reconciles teardown synchronously with `systemctl stop <unit>` so inherited
`--pipe` descriptors and sandbox descendants cannot outlive the stop; if
teardown cannot be confirmed (`exit 0` or unit-not-loaded `exit 5`), the
outcome fails closed as `Outcome::Interrupted` with unknown termination.
Worker `bwrap` argv and `Compatibility::workspace_provider_identity` remain
identical to direct-child launch.

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
`revision_not_admitted` with `dispatched:false`, after the fence record and
before any binding or intent; see [attempt seam](agent-revisions.md#attempt-seam).

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
before this change: `run` and `recover` never re-execute or recover it. Its
retained state is left as it was; only when its attempt directory holds no
execution intent does the refusal write the closure marker described below.
Retained attempts must be reconciled by their own coordinator, and new work
must be prepared again and dispatched as a new attempt.

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

Attempt ids must be unique across the work items of one state root: the
attempt directory is keyed by attempt id and fence only. Kiln's attempt id is
the BLAKE3 of the work's canonical key, ordinal and fence, so its ids never
collide. A request that finds another work's closure marker, intent or
pending admission in its attempt directory is refused as `crossed_attempt`: it
can never create its intent at that path. Its `dispatched` is `true` exactly
when that directory holds an execution intent of any request, which may have
run, and `false` for a closure marker or no intent.

Before the binding and intent, a run records its attempt at
`works/<blake3(work_id)>/<fence>`. A request whose fence is lower than one
already recorded for its work, or that names a different attempt at a
recorded fence, is refused. A request whose bytes differ from an existing
intent of its work is refused as substituted, and one whose attempt-fence was
closed by a refusal or recovery of other request bytes of its work as
`attempt_closed`. Each refusal prints one
`lattice.implementation-attempt-refusal.v1` record and the command exits
non-zero:

```json
{"schema":"lattice.implementation-attempt-refusal.v1","reason":"stale_fence","work_id":"...","attempt_id":"...","fence":7,"dispatched":false}
```

`dispatched` says whether a worker step of the refused attempt-fence (its
attempt and fence) may have started, not whether this request dispatched
anything. `dispatched:false` means no worker step of this attempt-fence has
started or can start in this state root, because a refusal decided before
this request's own intent writes the attempt-fence's closure marker under the
per-work lock; if another request's intent exists, the refusal is
`substituted_request` with `dispatched:true`. `dispatched` is `true` whenever
the attempt directory holds an execution intent other than a closure marker,
so a coordinator must never settle such an attempt as refused before any
effect or run it again.

Fencing is atomic per work item within one state root. Each work's fence
records live in the directory `works/<blake3(work_id)>` inside the state root,
and every process that writes there, or writes an intent of that work, first
takes an exclusive `flock(2)` lock on that directory (`std::fs::File::lock`).
Waiters block. The lock never expires, and the kernel releases it when the
holder closes it or exits. While holding it, a first run:

1. closes a session pin that an earlier admission of the same work left
   without its intent (below); another work's record is `crossed_attempt`;
2. reads the attempt directory's intent store: an execution intent of the
   same request bytes is reconciled as a duplicate `run`, one of other bytes is
   `substituted_request` (`dispatched:true`), and a closure marker is the
   closed attempt (`attempt_closed`, or `recover`'s report for its own marker);
   another work's marker or intent is `crossed_attempt`; nothing is admitted;
3. checks the lineage gate and the repair ports, which write nothing;
4. checks its fence and records its attempt;
5. with `--revision-store`, writes its write-ahead admission record, then
   admits its agent revision session, pinning it when managed;
6. persists its execution binding and candidate store;
7. rechecks that the record at its fence still names its attempt and that no
   higher fence is recorded, creates its execution intent and removes the
   write-ahead record, which the intent supersedes.

Before the lock and before step 1, a run checks the request, the workspace
boundary and the bounded-step timeout. Only `run` admits an attempt's own
session: `agent-revision admit-session` refuses it before any state change
(see [agent revisions](agent-revisions.md#attempt-seam)).

A typed refusal at steps 3 to 7 (`lineage_blocked`, `lineage_exhausted`,
`repair_ports_unavailable`, `repair_authority_denied`, `stale_fence`,
`crossed_attempt`, `revision_not_admitted`) is settled before the lock is released: the run
exclusively creates the attempt-fence's closure marker,
`closed-before-execution:<request identity>:<blake3(work_id)>`, the same
content recovery writes, then prints the refusal with `dispatched:false`.
Markers written before markers named their work
(`closed-before-execution:<request identity>`) still read as closed and are
treated as this work's. Such a refusal writes the state root, the per-work
lock directory, the attempt directory and the closure marker, plus anything
the earlier steps wrote (the fence record for `revision_not_admitted`), and
nothing else. A session the run pinned is closed on every path that ends
before its intent, before the refusal closes the attempt-fence; a session
another run pinned is never closed. The lock is taken before the agent
revision store is opened and never the other way round.

The write-ahead admission record, `admission-pending.json`
(`lattice.implementation-attempt-pending-admission.v1`), holds the BLAKE3 of
the work id (`work`), the request identity, the session and the lifecycle
subject (`definition`, `revision`) about to be pinned, and is created once
before `seam::admit` runs. It is removed when the admission is refused, when
the pin predates it, when the run abandons its pin, and once the intent
exists. A process lost after the pin and before the intent leaves it behind.
Every path that may write a closure marker reconciles it first, under the
work's lock and before anything else: a later run, recovery, pre-intent
refusal or superseded request's refusal of the same work closes the session
of exactly the recorded subject through the seam, leaving a session pinned to
another revision untouched, and only then removes the record. The record is
removed without closing only when its own request's execution intent exists,
which then owns the pin; a closure marker or another request's intent does
not. A request of another work leaves the record untouched and is refused as
`crossed_attempt`, creating no intent, marker or pin, with `dispatched` read
from the attempt directory as above. Recovery then
closes the attempt as before; a run proceeds or refuses as before. A record of
this work without its `--revision-store` is an error.

It releases the lock before any worker effect. A repair step takes the same
lock to recheck the fence before it writes `repair/step-N.intent`. Recovery
takes it to check for a crossed attempt and to create its closure marker. So
in one state root, a refusal with `dispatched:false` means that attempt-fence
has no execution intent and no request can create one there. Two state roots
share no lock and no records, so this guarantee does not span them; Kiln
remains the fence authority across them.

| `reason` | `dispatched` | When |
|---|---|---|
| `stale_fence` | `true` when the refused attempt's directory holds an execution intent; otherwise `false`, after a first run writes its closure marker; read under the fence lock | A higher fence of the work is already recorded |
| `crossed_attempt` | Same as `stale_fence`: `true` exactly when the shared attempt directory holds an execution intent of any request, including another work's; `false` for another work's closure marker or for another work's pending admission without an intent; nothing is written | Another attempt is recorded at this work and fence, or the attempt id and fence belong to another work in this state root |
| `substituted_request` | `true` | The attempt directory holds the intent of other request bytes not shown to be another work's, which may have executed; this request dispatched nothing and admitted nothing |
| `attempt_closed` | `false` | A refusal or recovery of other request bytes of this work closed this attempt-fence before any execution; no intent can be created for it |
| `profile_revision_superseded` | `true` when the attempt directory holds an intent other than a closure marker; otherwise `false`, after the closure marker is written | The envelope's revision is not the current framing; the attempt is never executed or recovered |
| `revision_not_admitted` | `false` | With `--revision-store`, the managed definition admits no new session of this revision, or the named `workflow_agent` is unknown, binds another profile revision or is a plain agent bundle; after the fence record only the closure marker is written ([agent revisions](agent-revisions.md#integration)) |
| `revision_blocked` | `false` | With `--revision-store`, the managed session was blocked right before spawn; the worker never started |
| `export_revoked` | `true` | `export-candidate` of a managed attempt that executed, after its session, pinned revision or qualification was revoked; nothing is exported |
| `lineage_blocked` | `false` | The request's Kiln budget lineage reports a blocker or serving backpressure; only the closure marker is written ([repair and lineage](#bounded-same-attempt-repair-and-budget-lineage)) |
| `lineage_exhausted` | `false` | The request's Kiln budget lineage cannot cover the first step; only the closure marker is written |
| `repair_ports_unavailable` | `false` | The request has a repair policy but no worker and check ports: an embedder supplied none, or the standalone CLI was given no `--repair-checks` catalog or one it does not admit (then with `check_catalog`); only the closure marker is written ([standalone operator repair](#standalone-operator-repair)) |
| `repair_authority_denied` | `false` | The standalone CLI's operator `effect_authority` profile does not grant the exact repair worker and check effects of this request, with `authority_refusal` (`unavailable`, `invalid`, `expired` or `not_granted`); only the closure marker is written and nothing is launched |
| `repair_outcome_unknown` | `true` | `export-candidate` of a repair attempt that executed but retains no `repair/disposition.json`, such as a controller lost between a step record and the disposition; the outcome stays unknown and nothing is exported ([repaired candidates](#exporting-repaired-candidates)) |
| `repair_not_candidate` | `true` | `export-candidate` of a repair attempt whose retained disposition is not `correct_candidate` or `repaired_candidate`; nothing is exported |
| `repair_disposition_invalid` | `true` | `export-candidate` of a repair attempt whose retained disposition is malformed, belongs to other request bytes, or contradicts this attempt, fence, envelope, execution binding or its own revision and step records; nothing is exported |

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
`closed-before-execution:<request identity>:<blake3(work_id)>` marker and prints
`status:"not_started_closed_by_recovery"` with `result:null`. A racing or later
`run` of that attempt can no longer create its intent, so it never dispatches.
Cancelling a closed attempt reports `not_started:true`, and `inspect` reports
no worker instance (`instance: null`) for an attempt this request closed,
whether recovery or a refusal before its intent closed it. A `run` of the
request that wrote the marker reconciles it like `recover`. Every observation, including
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

`export-candidate` reads only a complete retained result, or for a repair
attempt a passing retained disposition
([exporting repaired candidates](#exporting-repaired-candidates)). It rechecks
the frozen request, native receipt or explicitly admitted unchanged
observation, workspace identity, and retained changed-file bytes before
exporting. The output parent
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

Without embedder ports the standalone CLI uses its operator ports, only for a
catalog and profile that admit them
([standalone operator repair](#standalone-operator-repair)); otherwise it
refuses such a request as `repair_ports_unavailable` or
`repair_authority_denied`. An embedder passes `Ports::repair` to
`run_command_with_ports` with a `WorkerPort` (one step in the attempt
workspace), a `CheckPort` (one declared check of one candidate revision) and
a clock; the loop runs synchronously on the calling task. Both ports receive
a deadline, the time left before `max_elapsed_ms`, and must stop their work
by it. The lineage gate and the repair ports are checked under the per-work
lock before the fence record, so a first run refused as `lineage_blocked`,
`lineage_exhausted` or `repair_ports_unavailable` writes only its
attempt-fence's closure marker and the directories that hold it and the lock
(see [coordinator transport](#coordinator-transport)). After the usual
fence, revision session, binding, intent, revalidation and revision permit,
the shell:

1. re-observes the workspace and refuses it unless it equals the admitted
   `base_identity`;
2. asks the pure loop before every spawn, which stops on cancellation, an
   unreadable clock, the elapsed bound, the iteration bound, the aggregate
   budgets and the lineage, in that order;
3. under the fence lock, rechecks that the attempt is still recorded at its
   fence and that no higher fence is recorded; with the standalone operator
   ports, rechecks the operator's grant for every step
   ([standalone operator repair](#standalone-operator-repair)); for every
   step after the first, re-observes the execution binding and the managed
   revision through the same checks as the pre-spawn revalidation and
   revision permit; then writes `repair/step-N.intent` once before the
   worker effect. A failed fence check ends the loop as `fence_superseded`;
   a binding that is no longer current, a blocked managed session, a
   revision seam that cannot answer or a withdrawn operator grant ends it as
   `authority_withdrawn` with its `cause` (`execution_binding`,
   `managed_revision` or `effect_authority`). That step never spawns. Both are post-dispatch
   outcomes, retained and printed like any other disposition, because the
   attempt's intent exists and earlier steps may have had effects. They are
   never refusals, never report `dispatched:false` and write neither
   `binding-revalidation.json` nor `revision-block.json`; the observation's
   `authority_withdrawn` field names the step and carries the binding's
   revalidation record or the seam's refusal. The worker's deadline is read
   from the clock only after the lock is released, immediately before the
   worker is called: if waiting for the lock reached the elapsed bound the
   loop ends as `budget_exhausted` (`elapsed`), and if the clock was lost as
   `uncertain` (`clock`), with the step's intent retained and its worker
   never called;
4. re-reads the clock after the worker returns and charges the step. A
   cancelled or unknown worker outcome ends the loop before any proposal is
   considered; a step that returned at or past the elapsed bound is
   `budget_exhausted` (`elapsed`), an unreadable clock is `uncertain`
   (`clock`), and a step whose charges already exceed a policy budget or the
   lineage remainder is `budget_exhausted` naming that bound. None of these
   steps is checked;
5. captures the workspace after an exited step. A tree equal to the parent or
   the base is no candidate; otherwise a native receipt against the admitted
   base and a candidate revision binding envelope, attempt, fence, iteration,
   parent, candidate tree, execute bits (`mode & 0o111` of every executable
   file) and diff are derived on the host
   (`lattice.implementation-attempt-candidate-revision.v2`). The revision
   record also retains its execute bits and the base digest (length and
   BLAKE3) of every changed path. The worker's exit status, files and claims
   are never check results;
6. invokes every declared check against that exact revision. Each check runs
   against its own copy of the captured candidate under
   `repair/check-N-<check>/`, written from the host snapshot with every write
   permission removed, never against the worker workspace. The copy records
   the type and permission bits it sealed for every file and directory. After
   the check the copy is captured again: if its bytes no longer match the
   candidate, or any sealed path changed type or mode (a newly executable
   script, a writable directory), the evidence is rejected as
   `candidate_drift` and no further check runs. The copy is then removed. The
   observations are classified: a skipped, missing, duplicated, undeclared,
   reconfigured, crossed or self-approved (run by the worker profile itself)
   check, or one from any executor other than `check_executor`, rejects the
   evidence. Each check's deadline is read from the clock after its copy is
   written, immediately before the check is called; a reached bound or an
   unreadable clock then stops the loop without calling it;
7. before `correct_candidate` or `repaired_candidate`, re-reads the clock and
   compares the charged totals, checks included, with every policy budget and
   the lineage remainder; a pass past the elapsed bound or over any of them is
   `budget_exhausted` naming the bound, and an unreadable clock is `uncertain`
   (`clock`);
8. writes `repair/step-N.json` (`lattice.implementation-attempt-repair-step.v2`:
   the step, its report, its candidate revision, every check observation, the
   integrity of every check's copy and the verdict) once and repeats only for
   failures inside the admitted classes.

The disposition is one of `correct_candidate`, `repaired_candidate` (with
`repairs`), `budget_exhausted` (with its `bound`: `iterations`, `elapsed`,
`model_requests`, `tool_invocations` or `lineage` and its dimension),
`cancelled`, `uncertain` (`worker_outcome`, `check_outcome` or `clock`),
`not_repairable` (with the classes), `checks_rejected` (with the defect),
`lineage_blocked`, `kiln_owned`, `fence_superseded` or `authority_withdrawn`
(with its `cause`). A worker that exits and proposes `changed_work` or a
`new_semantic_attempt` ends the loop as `kiln_owned`; with a cancelled or
unknown outcome the proposal is ignored and the loop ends as `cancelled` or
`uncertain`. A step whose binding names other work, version or task, or
another attempt, fence or envelope, is refused before it is recorded. Repair
steps receive the parent revision and the admitted failure classes only, never
a transcript or check output. The
`lattice.implementation-attempt-repair-observation.v2` record (envelope,
request and worker-request identities, execution binding, policy and lineage
identities, disposition, step count, revisions with their receipts, execute
bits and changed-path base digests, usage, draws and unaccounted dimensions,
`accepted:false`, `integrated:false`, the dispatch plan when present and
`authority_withdrawn` when that is the disposition) is written once to
`repair/disposition.json` after the attempt's `Scope::Attempt` usage
observation, and printed. Both records are typed: an unknown field, a missing
field or a nullable field left out is refused. Recovery and a duplicate `run`
print the retained disposition, also a first-schema
(`...repair-observation.v1`) one when its request identities, envelope,
policy, execution binding, step count and `accepted`/`integrated` fields name
this attempt; any other first-schema record is malformed or crossed, read the
same way by recovery, export and inventory. Without a disposition the outcome
is unknown and nothing runs again. No disposition is acceptance: Kiln #46 keeps independent
verification. `export-candidate` exports the candidate of a passing
disposition and `inspect` reports a disposition whose records prove its
workers stopped ([exporting repaired candidates](#exporting-repaired-candidates)).

Usage is attributed per phase (`preparation`, `edit`, `check`, `repair`) with
the existing `UsageProvenance`. A worker step whose broker evidence is
complete in the step's own request-index range is charged exactly the model
requests it dispatched there, reported with provenance `host_measured`
(`WorkerPort` reports carry this as `observed_model_requests`; see
[brokered evidence](worker-serving-binding.md#brokered-evidence-without-a-serving-declaration)).
Step `n` owns indices `[(n - 1) * max_requests, n * max_requests)`. Otherwise
a worker step is charged the larger of its reservation and its reported
count per dimension, or the reservation when the count is unknown
(`provenance:"unknown"`, counts `null`); an intent without a terminal record
or an `uncertain` request in the step's range keeps that reservation. The
standalone OMP port reports no tool invocations, so they are charged at
their reservation.
Checks are host executions with a known count and no model requests. Failed,
cancelled and uncertain steps are charged like any other.

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
| `remaining` | the account's `available` `Amounts`, at most `MAX_DIMENSIONS` (16), keyed by Kiln `Alias` names: any non-empty string of at most `DEFAULT_MAX_ALIAS_BYTES` (256) bytes, such as `gpu-seconds` |
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
within the remainder in every one of its dimensions. Other dimensions, any
name Kiln's `Alias` admits, are accepted, never drawn and listed in
`unaccounted_dimensions`: their usage stays unknown to Kiln. A single-shot
attempt with a lineage reports `budget_lineage` in its observation with
`tool_invocations` unaccounted and one model-request draw: exactly the
requests its complete broker evidence proves, as `host_measured`, or one
broker ceiling as `unknown` when any intent lacks a terminal record, a request
ended uncertain, the evidence conflicts or none was retained. The draw is
retained write-once as `lineage-draw.json` before the attempt result is
published; recovery replays it as retained, and an attempt without one
reports the ceiling, so a retained draw is never lowered. The lineage is a
frozen snapshot: Lattice does not observe Kiln's current ledger.

Qualification is fixture-level: the tests in
`src/implementation_attempt/cli/tests/repair.rs` drive the real `run` and
`recover` shell with a fixture worker, check port and clock, and those in
`src/implementation_attempt/cli/tests/repair/operator.rs` drive the
operator ports with a scripted fake OMP executable and catalog checks
without bubblewrap. No real OMP, model endpoint, sandboxed check or Kiln
lineage has been exercised.

#### Standalone operator repair

`lattice implementation-attempt run --repair-checks CATALOG` runs a request
with a repair policy through the operator ports when no embedder supplied
repair ports. They grant nothing themselves and are never used for a request
without a policy, which keeps its bytes and behaviour and reads no
configuration or catalog.

- **Worker:** each step launches the pinned OMP once through the single-shot
  route (`candidate_worker::omp::run`): the same re-admitted workspace
  boundary, bubblewrap provider, byte pins, reviewed argv and attempt-local
  broker with the host-only `env` credential. The step's OMP base is the
  expected tree: the admitted base for the first step, then the workspace
  the previous exited step left. The launcher refuses any other workspace,
  so a workspace changed between steps blocks the launch and the step is
  `unknown`. A repair's prompt is the admitted prompt plus the parent
  revision, "the previous step produced no new candidate" after a
  `no_candidate` failure, and the other failed check classes only. Its
  deadline is the loop's remaining time rounded up to whole seconds, so when
  the elapsed bound is the limit a worker that runs to it returns past the
  bound and the step is `budget_exhausted` (`elapsed`), never checked. When
  the admission's expiry caps the deadline below that (`deadline_capped` in
  the step record), a stop at the capped deadline is `unknown`, never
  checked. A persisted cancellation request is polled during the step. Raw
  events and the launcher observation are written once as
  `repair/omp-step-N.jsonl` and `repair/omp-step-N.json` before the step
  record.
- **Outcomes:** `exited` for an observed exit (a completed candidate or no
  change is status 0) and for a host stop at an uncapped deadline or the
  output limit (status 137); a signal death inside the sandbox is an observed
  exit with bubblewrap's `128 + N` status. `cancelled` is a cancelled step,
  and `unknown` is anything else: a launch blocked before spawn, a stop
  without an exit status (bubblewrap itself killed), a write outside the
  boundary, an interrupted or unknown teardown and an incomplete OMP
  terminal stream. `unknown` ends the loop as `uncertain` (`worker_outcome`),
  which inventory never reports as stopped, and nothing is retried. A
  launcher that panics is a lost controller: the step intent stays without a
  record, and recovery reports the outcome unknown without a second launch.
  A `SIGINT` or `SIGTERM` to `run` ends the controller the same way, unlike a
  single-shot run; use `implementation-attempt cancel` for a clean stop.
- **Checks:** the catalog (`lattice.implementation-attempt-check-catalog.v1`,
  at most 64 KiB, 16 entries sorted by name, 32 arguments of at most 4 KiB
  each) is operator input read outside the attempt workspace:

  ```json
  {"schema":"lattice.implementation-attempt-check-catalog.v1",
   "sandbox":{"path":"/nix/store/...-bubblewrap/bin/bwrap","blake3":"<64 hex>"},
   "checks":[{"name":"unit","program":{"path":"/nix/store/.../bin/unit","blake3":"<64 hex>"},
              "args":["--all"],"failure_class":"test"}]}
  ```

  Every program and the sandbox must be an absolute Nix store path whose
  bytes have the pinned BLAKE3; the catalog is opened without following
  links or blocking and must be a regular file. A check's `configuration` is
  the domain-separated BLAKE3 of its entry and the policy's `check_executor`
  that of the executor profile (schema, route, sandbox pin and the 1 MiB
  output bound). The pins cover the entry executable's bytes only: inside
  the sandbox, interpreters and tools it resolves through shebangs or
  `PATH=/usr/bin:/bin` come from the host's read-only system mounts (`/usr`,
  `/bin`, `/lib*`, and on NixOS `/run/current-system/sw/bin`), which the
  shared candidate-worker sandbox provides and no identity frames, so a host
  upgrade can change what a check runs. Pin a self-contained store program
  (for example a store script with absolute store shebangs and paths) to
  avoid that. Before each check the operator's grant and the admission are
  rechecked and a persisted cancellation request is read; any of them stops
  the check unrun with an unknown outcome. Each check runs once through the
  candidate worker's generic bubblewrap route with its read-only copy as
  root, the request's read paths that exist in the copy mounted read-only,
  no write path, a cleared environment, a private network, no broker or
  credential, stdout and stderr bounded to 1 MiB each, and a deadline within
  the admission's remaining seconds. An exit observed on an unchanged copy
  decides it: zero passes and a status from 1 to 125 fails with the entry's
  class. A cannot-execute (126), not-found (127) or signal (`128 + N`)
  status and anything else is an unknown outcome, so the loop ends
  `uncertain` (`check_outcome`) instead of spending repairs on an
  infrastructure fault; a bubblewrap setup failure exits 1 and cannot be told
  apart from a failing check. A missing catalog is `repair_ports_unavailable`;
  one that is unreadable, not a regular file, oversized, malformed, unpinned,
  byte-mismatched, inside the workspace or not exactly the policy's is
  `repair_ports_unavailable` with `check_catalog`.
- **Authority:** the `effect_authority` profile of the application
  configuration (`--config`) must hold both tuples below, with
  `workflow_revision` `blake3:<worker_profile>` and the operator repair
  machine identity; `prepare` of a request with a policy prints these
  requirements as `repair_authority`, in the form `authority inspect` prints,
  for the [operator-profile steps](choregraph-effect-authority.md#standalone-operator-profile):

  | Effect | `node_id` | `operation` | `resource` | `ability` |
  |---|---|---|---|---|
  | OMP repair worker | 1 | `lattice/agent` | `lattice://agent/1` | `agent/execute` |
  | Catalog checks | 2 | `lattice/shell` | `lattice://process/2` | `process/execute` |

  Without both, the run is refused as `repair_authority_denied` before the
  fence record, with no step intent. Before every step's intent (under the
  fence lock), before every revision's checks and before every check, the
  same profile is evaluated once again and must keep its admitted identity
  and both grants, and the admission must not have expired. A lapse before a
  step or before a revision's checks ends the loop as `authority_withdrawn`
  with cause `effect_authority`, retaining the recheck's evidence; a lapse
  between individual checks leaves the later checks unrun, and the loop ends
  `uncertain` (`check_outcome`). A persisted cancellation request found when
  the checks end stops the loop as `cancelled`, as it does during a worker
  step.

Every refusal here closes the attempt-fence for good, like the other
pre-intent refusals: a configuration that does not load, an unreadable or
mistyped `--repair-checks` path or a profile that cannot be evaluated
leaves only the closure marker, and the attempt must be dispatched again
as a new attempt. Export, inventory and recovery read these attempts like
any other repair attempt. The tests run the fake OMP and the checks without
bubblewrap or broker, so `PinnedOmp` and the sandboxed check route
(`checks::Sandboxed`) are not exercised by any test.

#### Exporting repaired candidates

A repair attempt has no native attempt result, so `export-candidate` and
`inspect` read its retained repair records instead, and act only on the
complete set: the typed v2 disposition of the exact request bytes naming this
envelope, persisted execution binding, policy, lineage and dispatch plan, with
`accepted:false` and `integrated:false`, and the typed intent and record of
every step it counts, each this attempt's step at its iteration, within the
admitted policy's iterations. No single recorded field, a kind, a verdict or
a receipt's own claims, is trusted alone.

Export first runs the same checks as a single-shot export: the request,
envelope and binding validation, the persisted intent of the exact request
bytes, the attempt-fence directory and output parent, and, for a managed
attempt, the revision seam, which refuses a revoked session, pinned revision
or qualification as `export_revoked`. An attempt refused or blocked before
dispatch has no candidate. Then:

- no retained disposition, for example after a controller was lost between a
  step record and the disposition, is `repair_outcome_unknown`; the outcome
  stays unknown and is never guessed;
- an unreadable, malformed, incomplete, first-schema or crossed record set is
  `repair_disposition_invalid`;
- any disposition other than `correct_candidate` and `repaired_candidate` is
  `repair_not_candidate`;
- a passing disposition is replayed in the pure core
  (`repair::exportable`). Its repairs must fit the admitted policy and the
  global bound of 8, and the step count must be `repairs + 1`. Every step must
  be exited, with the parent and failure classes the loop would have given it.
  Every recorded revision must re-derive from this envelope, attempt and
  fence, its iteration, parent, candidate tree, execute bits and diff, and
  carry a produced receipt of this work, attempt, fence, workspace and base
  whose changed paths are exactly its retained base digests. Every step's
  verdict is re-derived from its check observations and the integrity of each
  declared check's copy (`repair::checks::classify`); each step before the
  last must have failed only with admitted classes, and the last must pass
  with the named revision. Anything else is `repair_disposition_invalid`.

The workspace must then still capture to the final revision: its execute bits
must equal the retained ones the checks' copies were sealed with, and the
whole receipt, changed paths included, must re-derive from the captured
candidate and the retained base digests of the changed paths, so the base
identity must also be the admitted one. Capture admits only regular files, so
no other file type can appear. Otherwise nothing is exported. The output is
the single-shot layout: the workspace snapshot, `host-receipt.json` with the
re-derived final receipt and `native-observation.json` with the retained
disposition, replayed exactly. All three refusals report `dispatched:true`.
Single-shot attempts are exported exactly as before.

[Worker inventory](worker-inventory.md#instance-observation) reports a repair
attempt as `terminal_observed` with the disposition's `kind` only when the
complete records agree with the kind and prove that every worker step the run
started has stopped:

- the kind must agree with what the records re-derive: every recorded verdict
  is its re-derived one, `correct_candidate` and `repaired_candidate` replay as
  for export, `not_repairable` is the last step's re-derived failure outside
  the admitted classes (or `no_candidate`), `checks_rejected` its re-derived
  defect and `uncertain` (`check_outcome`) its re-derived unknown outcome;
  otherwise the record is an error;
- every recorded step reports its worker `exited` or `cancelled`. So
  `correct_candidate`, `repaired_candidate`, `budget_exhausted`, `cancelled`,
  `uncertain` (`check_outcome` or `clock`), `not_repairable`,
  `checks_rejected`, `lineage_blocked`, `kiln_owned`, `fence_superseded` and
  `authority_withdrawn` are terminal when their records show this, and
  `uncertain` (`worker_outcome`), whose worker outcome, including an unknown
  teardown, is unknown, is never terminal;
- a step intent after the counted steps without a record is a worker that
  was never called only for `budget_exhausted` (`elapsed`) and `uncertain`
  (`clock`), which the loop decides after the intent and before the call;
  with any other disposition that worker may have run.

Anything else stays `dispatch_recorded_outcome_unknown` with the
`dispatch_outcome_unknown` reason, as does an attempt without a disposition or
with a valid first-schema one of this attempt. A malformed, crossed or
contradicted record set, first-schema ones included, is an error, never
terminal.

The shell tests in `src/implementation_attempt/cli/tests/repair/export.rs` and
`src/implementation_attempt/cli/tests/repair/forged.rs` drive fixture repair
runs through `run`, `export-candidate` and `inspect`, including a revision
revoked after the run and records and workspaces changed after it; the pure
tests in `src/implementation_attempt/repair/exportable/tests.rs`,
`src/implementation_attempt/repair/retained/tests.rs` and
`src/candidate/digests/tests.rs` cover the replay, the stop rule and the
receipt re-derivation. The same non-claims apply, and anyone who can rewrite
the private state directory can still forge a consistent record set. An
export is not acceptance: Kiln #46 still verifies the exported candidate
independently.


The scoped broker extends the previously published Lattice worker route at
`06e551e5b08ce78a46d03af5b5870ef1de81ac00`. Consumers must pin the actual source
archive/executable identities. Kiln #46 still owns independent verification and
canonical CI acceptance; its current contract admits supplied observations but
does not execute the verification profile. A live OMP/coordinator pilot and
external current-grant/worker/compute/service qualification remain separate.
