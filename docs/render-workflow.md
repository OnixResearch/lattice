# Artifact-bound tiled rendering (opt in)

Lattice consumes Aspen's canonical `molten-render-pilot` manifest, `prepare`,
`content_id`, `assemble` validation and **`verify_final`**, plus the actual
`molten-batch-core` observation/fence types. It does not define a second render
protocol, tile scheduler or renderer. The exact producer cohort is
`afc8308bff9a8c50b2315cb4a17c9454252d7c45` from anonymous public HTTPS.

## Acquisition and build boundary

`vendor-cargo-deps/aspen-render/source/` is the byte-exact sparse contract closure
(core, batch-core, codec, render). `SOURCE.json` retains the public archive URL,
revision, archive BLAKE3, every selected file BLAKE3 and license. Its sparse
`source/Cargo.toml` is explicitly Lattice-owned metadata, not an edited producer
manifest. Refresh with `python3 scripts/import-aspen-render.py`; a supplied local
archive must have the same public archive identity. Changed imported files are
never overwritten by the importer. Production Cargo dependencies use that
repository-owned closure, never `../aspen`.

The runtime adapter launches the selected, content-measured **real
`molten-orchestrator` CLI**, whose existing authenticated iroh boundary owns
publication, submission, placement, retry, progress and worker stop. It requires
an absolute executable and its `operation::identity::executor_file` identity;
there is no `PATH`, local renderer or native/Wasm fallback. The public fixture
acquires the full same producer archive separately to build that CLI and its
import-free component with the producer's actual pinned Nix/component rail.

Aspen's ABI remains `run(list<list<u8>>) -> list<u8>`. Lattice's component ABI is
not interchangeable. No changes are made to Lattice's one-active-local-component
rule: the render operation is a bounded native process consumer, not a component
host. All remote tile fan-out and final assembly execute inside Aspen's batch.

## Immutable admission and actual execution ownership

`render::Request` binds the canonical manifest, exact producer executable,
controller public key/numeric address, cluster/generation/policy, component,
logical execution, owning attempt/fence, original budget/deadline, current-owner
file and private journal root. The producer manifest binds scene/schema, camera,
renderer/profile, dimensions, samples/bounces, RNG/seed, partition, encoding and
resource/retry bounds. The entire job, **including assembly and its literal
assignment**, is admitted by producer `prepare` before publication. Reservation
ceilings include every tile plus assembly at every allowed producer attempt.

`lattice render prepare --request REQUEST.json --out NEW_ABSOLUTE_DIR
--lattice ABSOLUTE_LATTICE_BINARY --repository ABSOLUTE_REPOSITORY [--method
advance|observe|cancel|supersede]` produces an inert catalog, instance, invocation
plan, canonical manifest/job and exact Nickel graph. The operation instance
freezes both the binding path **and canonical binding identity**. Planning does
not grant effects; import, inspect and explicitly authorize this graph using
[the existing effect-authority profile](choregraph-effect-authority.md).

The closed operation runtime is `process/lattice-render-v1`, type
`molten.render`. Its four exact methods are mutations for capability planning;
none can borrow the automatic Cairn read-only authority. The ordinary configured
Choregraph/Basalt graph authority gates dispatch. The standalone read-only
`operation run`/operation-pack convenience route deliberately cannot grant this
mutation. No render-specific backend class or automatic operator grant is added.
The graph has cache disabled and no automatic retries.

At real Executor dispatch, the host binds its allocated workflow run, admitted
authority RunIdentity, node, workflow revision and invocation to the journal **before a possible batch
Submit**. A retained host-dispatch record is provenance, not a capability or
substitute for existing effect admission. The CLI entrypoint requires that exact
record; caller-chosen run numbers in a binding or argv alone are insufficient.
A later **Advance** from a crossed host run cannot take over that computation.
An explicit **Observe** operation may have a new observer run/plan, but preserves
the original compute owner in the accepted manifest and records the actual
observer context separately. It cannot Submit. Context-free
`process_request_plan` remains an inert inspection API; dispatch preparation and
classification require actual execution context for rendering. Direct standalone
render dispatch without owned context is unsupported and denied.

The selected current-owner JSON is supplied by the existing logical
execution/claim owner; render only reads it. It contains exact `execution`,
`attempt`, `fence`, `budget_identity`, and `current` fields. The adapter never
mints claims or advances fences. Private journal and authority roots and their
ancestry must be operator/service-owned, not writable by an untrusted workload.
File rechecks are local observations, not distributed atomic fencing or stronger
revocation freshness than the existing authority owner provides.

## Recovery and acceptance

The bounded create-only journal records intent, possible dispatch, ACK,
observations, cancellation intent and accepted artifacts, with file/directory
sync and one OS-held writer lock. Once Submit is possible, **no reopen or error
branch repeats Submit**. The local journal admits at most 512 records of at most
1 MiB each and one create-only download slot per accepted content ID; retention
exhaustion denies fresh activity rather than allocating unbounded storage or
resetting state. The producer's existing `run-id.v1(cluster,key)` permits
read-only lookup of that exact run after ACK loss. Missing controller/worker,
missing content, unsupported recovery or malformed observations retain Unknown;
they do not create a new key or reset the governing reservation/deadline. A
crash after durable intent but before the caller knows whether Submit occurred
can remain Unknown indefinitely if the producer has no such accepted run. This
is intentionally not an exactly-once guarantee.

Only authoritative `ProgressTask::Accepted` IDs are retained. Every accepted
snapshot must match exact cluster, generation, run and spec, complete expected
task inventory, task fence/epoch, canonical assignment/request/result identity and
the selected producer policy. A terminal phase or exit zero alone is insufficient.
Fetched bytes must match **content-manifest identity** (not only raw BLAKE3).
Canonical producer assembly validates every tile's assignment, coordinates,
length, encoding, digest, pixels and exact coverage. Its result must equal the
accepted final envelope, which is independently passed through `verify_final`.
Missing, duplicated/conflicting, crossed-attempt, rehashed wrong-pixel, stale,
unexpected or partial results cannot complete the operation. Exact accepted
snapshots/maps apply once; repeat observations cannot inflate tile completion.

A complete journal publishes `accepted-manifest.json`, accepted tile/final
objects and a verified `image.ppm` presentation. Publication alone is not workflow
completion: the operation classifier rechecks currentness, durable completion,
host provenance, accepted manifest identity and all retained bytes. A stale
attempt/supersession tombstone blocks late completion. A cancellation intent
always blocks late result acceptance, even if the producer concurrently succeeds.
`cancellation_requested` is distinct from `stopped`; only a producer-observed
terminal cancellation establishes the latter. Killing the local observer or
client does not stop remote execution. Original deadline expiry prevents new
submission/acceptance and requests cancellation; bounded read-only reconciliation
and cancellation do not reset computation budget.
Aspen's current `JobSpec`/`ExecutionRequest` has per-attempt `timeout_ms`, fuel
and retry bounds, but no job-level absolute-deadline/aggregate-budget/current
Lattice-fence admission port. A disconnected observer therefore cannot prove
remote deadline enforcement or stop. That durable producer/managed-budget-owner
API is the precise prerequisite for detached aggregate-deadline enforcement;
the consumer does not invent it or equate local deadline expiry with remote stop.

Receipts distinguish `new_computation_requested`, `restored_workflow_state` and
`reused_committed_artifacts`. Budget values are reserved ceilings. Local adapter
wall time is measured independently. Remote CPU/fuel/memory/cost remain null with
an explicit reason: producer `execution-result.v1` has no usage telemetry. Neither
elapsed client time nor reserved fuel is substituted for measured remote usage.

## Retained qualification and explicit non-claims

From the repository root, build the packaged runtime before qualification:

```sh
nix develop -c cargo test --locked --test test_render_workflow
nix build .#lattice
nix-shell --pure examples/render-workflow/shell.nix --run \
  'python3 scripts/qualify-render.py --retain /ABSOLUTE/NEW/PRIVATE/RETAINED-DIR --lattice "$(realpath result/bin/lattice)"'
```

The existing executor identity boundary admits at most 256 MiB per executable.
Large debug binaries can exceed that ceiling and are rejected before submission;
select the packaged runtime rather than weakening the bound.

The last command is explicit opt-in to isolated loopback controller/worker
processes and exact local operator grants for the generated test graphs. It
acquires and identity-checks the immutable public producer source, uses the
producer's Nix shell/build rail, runs the **actual admitted Lattice graph**, then
compares a second real run under the same backend/host/component/profile. It
retains every tile/final accepted ID and exact verified PPM comparison, graph,
catalog, operation plan, producer/runtime/component/WIT/Cargo/Nix/source identities,
raw receipts, command logs and measured/unavailable timings. This route uses a
closed builtin operation rather than an operation pack; its pack identity is
explicitly not applicable, never a fabricated pack lock. Caller-supplied Lattice
binary/source correspondence still needs the caller's build provenance.

By default the fixture additionally discards a real Submit ACK in the immutable
consumer fault rail, reconnects with Observe, kills/restarts an active controller,
kills/restarts an active worker plus controller after accepted progress, and kills
the owned local observer session before explicit active cancellation. Each requested interruption
must actually be observed or the fixture fails. Select controls with
`--controls lost-ack controller worker cancel`, or provide `--controls` alone for
only completion/repeat. The source-bound optional `route.fixture_fault` also
supports `after-intent`, `after-observation` and `before-publication`. It is a
conformance interruption, not a retry/repair mode. Fault choices change the exact
binding identity. No live mutable fault file can alter an admitted request.

Native consumer regressions use actual canonical producer functions to construct
bounded contract bytes and synthetic transport observations. They **do not**
qualify the real component, network or Lattice process route. Producer tests and
historical Aspen receipts are likewise not Lattice qualification. The process
fixture writes passing observations only after its actual assertions succeed.
The [earlier native observation](../.cairn/changes/bind-artifact-render-workflow/evidence/local-process-observation.json)
retains its original source/executable cohort. The separate
[published-runtime observation](../.cairn/changes/bind-artifact-render-workflow/evidence/published-runtime-observation.json)
records the packaged Nix runtime from published revision `3539ade3`, including
completion/repetition, lost ACK, controller/worker interruption and observed
terminal cancellation. Later edits or rebuilt artifacts require their own
execution; neither record silently qualifies a newer binary.

The qualified comparator here is exact accepted identity/PPM bytes on one actual
backend/profile/host. There is no qualified cross-backend comparator and no claim
of native/Wasm or CPU-architecture bit identity. Remote usage telemetry, durable
machine-loss storage and independently provisioned two-host execution remain
explicitly outside these local observations; no producer or Lattice receipt is
promoted into those claims. Review `.private` node state and source archives
before sharing anything. The fixture never deletes retained state, publishes
artifacts, changes issues or commits source.
