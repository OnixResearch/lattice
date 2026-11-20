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
agent definition and declares bounded task families and tools. It is part of
`worker_profile_identity` only when present, so existing requests keep their
revision. A malformed declaration is refused before any effect. `inspect`
prints `lattice.worker-inventory.v2`; see [worker inventory](worker-inventory.md).

Re-entry with the exact request only reconciles retained results and reports the
persisted binding identity; it never calls the compute port, rebinds or
releases compute. Changed requests and missing/incomplete evidence never
authorize another execution. Raw JSONL and normalization artifacts are
separately fsynced before the candidate result. Recovery bounds their reads,
rejects symbolic links, checks the raw digest against the owned stdout
observation, and independently repeats normalization. Missing, substituted, or
contradictory artifacts cannot become a published result. Version 2 removed the
old `network`/`network_namespace` fields rather than accepting shared-network
compatibility requests. Broker expiry cannot exceed the enclosing attempt
expiry. This shell creates no external authorization grant.

Cancel writes only an exact persisted request; the running owner observes it and
uses the existing process-group cancellation. Requested cancellation is not a
termination receipt. Controller loss or unknown teardown retains the workspace
and usage uncertainty rather than releasing it for a successor.

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


The scoped broker extends the previously published Lattice worker route at
`06e551e5b08ce78a46d03af5b5870ef1de81ac00`. Consumers must pin the actual source
archive/executable identities. Kiln #46 still owns independent verification and
canonical CI acceptance; its current contract admits supplied observations but
does not execute the verification profile. A live OMP/coordinator pilot and
external current-grant/worker/compute/service qualification remain separate.
