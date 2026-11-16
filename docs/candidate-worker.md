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
access. Network is private when `boundary.network` is false. With true it is
shared network access, **not endpoint-scoped egress enforcement**.

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
revision, an exact model, one non-command prompt, and the independently admitted
base snapshot identity. The executable must be an absolute path. The wrapper
owns argv; arbitrary caller argv, resume and interactive `/loop` are not admitted:

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

```sh
cargo test --locked --lib process::shell::failure_tests
cargo test --locked --lib candidate_worker
cargo test --locked --test test_store candidate_attempt
cargo test --locked --test test_local_process
# With LATTICE_BWRAP set to the reviewed provider executable:
cargo test --locked --lib candidate_worker::tests::bubblewrap_ -- --ignored
# Also supply LATTICE_OMP, LATTICE_OMP_REVISION, LATTICE_OMP_BLAKE3,
# LATTICE_OMP_BASE_URL and LATTICE_OMP_MODEL for the reviewed local endpoint:
cargo test --locked --lib candidate_worker::tests::omp_worker_produces_a_sandboxed_candidate -- --ignored --exact
```

These commands are verification instructions, not a report that they were run.
No live OMP build/provider/model qualification is claimed by source fixtures.
An admitted immutable OMP build with source provenance, measured provider policy
on the deployment kernel, endpoint-scoped model/network/credential capability,
and selected-route failure/cancellation evidence remain qualification
prerequisites. There is no credential delivery provider, independent acceptance,
publisher, merge authority, external exactly-once guarantee or production-readiness
claim in this contract.

## Coordinator-admitted implementation consumer

The separately selected `lattice implementation-attempt` commands consume the
existing immutable envelope and isolated OMP worker. They do not change the
guarded drain/review routes or acquire Kiln's global work authority:

```sh
lattice implementation-attempt prepare --request seed.json --output request.json
lattice implementation-attempt run --request request.json --state /var/lib/lattice/implementation
lattice implementation-attempt recover --request request.json --state /var/lib/lattice/implementation
lattice implementation-attempt cancel --request request.json --state /var/lib/lattice/implementation
lattice implementation-attempt export-candidate --request request.json --state /var/lib/lattice/implementation --output-parent /var/lib/lattice/candidates
```

The strict `lattice.implementation-attempt-run.v1` fields and canonical profile
codec are in `src/implementation_attempt/cli.rs`. Preparation reads the actual
workspace snapshot and emits snapshot/profile/envelope/request-byte identities;
it reports `admitted:false` and grants no authority. The installed operator
request binds current work/version/attempt/fence/source/spec/base and an expiry.
Only the trusted Kiln dispatcher may admit it through current claim/resource
grants. Model credentials and publisher authority remain unavailable.

A private one-shot execution intent is fsynced before the worker starts. Re-entry
with the exact request only reconciles retained results; changed requests and
missing/incomplete evidence never authorize another execution. Raw JSONL and
normalization artifacts are separately fsynced before the candidate result.
Recovery bounds their reads, rejects symbolic links, checks the raw digest against
the owned stdout observation, and independently repeats normalization. Missing,
substituted, or contradictory artifacts cannot become a published result.
A network-enabled request names a backend-owned `/run/netns/NAME`; execution
verifies the actual namespace identity. No namespace or egress policy is created
by this command.

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


The source additions are an explicitly unpublished local correction cohort over
`c6da59566a896f432f8c3a8f419033364e376cf2`, not a fabricated published revision.
The consumer must pin its exact source archive/executable identities. No live
OMP/model/kernel, cancellation campaign, GitHub effect or deployment is claimed
by these source and fixture changes.
