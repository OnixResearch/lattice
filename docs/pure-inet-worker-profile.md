# Pure interaction-net worker profile

This opt-in profile evaluates one immutable, finite, acyclic two-input
wrapping-`i64` program through the pinned Noble interaction-net worker. It uses the
existing operation-pack `bounded_process` route, typed operation planner, local
read-only authority, durable operation-run owner, process runner and
operation-data publication. It adds no runtime class, workflow node, scheduler,
reducer or fallback. Nothing selects it by default.

Tracking: [Lattice #14](https://github.com/OnixResearch/lattice/issues/14) and
[Lattice #22](https://github.com/OnixResearch/lattice/issues/22). Producer:
[Noble #37](https://github.com/OnixResearch/noble-lang/issues/37). Consumer:
[Mneme #5](https://github.com/OnixResearch/mneme/issues/5) and its
`add-optional-inet-compute` change.

## Producer pin

The flake input `noble-lang` pins the Noble source revision over anonymous HTTPS,
and Noble's own SSH-locked inputs are pinned to the same revisions over HTTPS. The
development shell and checks receive the built executable as `LATTICE_NOBLE_BIN`
(`noble-lang.packages.<system>.noble`). Noble publishes only an
`x86_64-linux` package; other systems omit the executable-backed checks rather
than substitute another worker.

`lattice::inet::report::PRODUCER_SOURCE_REVISION` must equal the revision the
pinned build declares in its report (`producer.source_revision`, status
`build-declared`). The report names no executable digest, so Lattice binds the
executable by its own measurement: the pack's content identity and the
operation runtime's executor identity.

## Closed vocabulary

The program artifact is the producer's own `ngraph-ir/0 ngraph-i64/0` graph file
and the input artifact is its `noble-ngraph-vectors/0` vector file. Lattice admits
only their canonical printed subset (`lattice::inet::text`):

- two `slot input i64` lines, each defined by exactly one `input` node;
- `const i64`, `i64.add`, `i64.sub` and `i64.mul` nodes whose operands are strictly
  earlier nodes (a forward reference is a cycle and is refused);
- exactly one `out` line, one vector row, and canonical decimals everywhere.

`i64.eq`, F32 operations, parameters, guards, facts, shapes, repeated outputs,
comments and noncanonical spacing or integers are refused before dispatch.
Programs that share a node are evaluated by the producer's `dup` agents, not
recomputed. JSON consumers carry inputs as
`{"schema": "lattice.pure-inet-inputs.v1", "inputs": ["<i64>", "<i64>"]}`; JSON
numbers are refused even when they fit exactly.

## Authoring and invocation

`lattice::inet::route::pack::author(destination, noble)` creates a new pack directory
containing the profile's one-type operation catalog and an exact copy of the
supplied executable; it never builds or downloads one.
`lattice::inet::route::instance(catalog, artifact_root)` authors the data-only
instance whose single `repository_root` target is the absolute directory holding
the immutable artifacts. The Noble source declares no license, so admission must
explicitly allow both `AGPL-3.0-or-later` (catalog) and
`LicenseRef-noble-lang-undeclared` (executable).

```sh
lattice pack lock --pack /abs/pack --confirm-trust-local \
  --allow-license AGPL-3.0-or-later --allow-license LicenseRef-noble-lang-undeclared \
  --destination /abs/pack.lock.json --confirm-create
lattice pack run --pack /abs/pack --confirm-trust-local \
  --allow-license AGPL-3.0-or-later --allow-license LicenseRef-noble-lang-undeclared \
  --lock /abs/pack.lock.json --instances /abs/instances.ncl \
  --instance pure-inet-artifacts --method evaluate \
  --arguments-json "$(cat arguments.json)" --confirm-read-only
```

`arguments.json` names each artifact file inside the root with its BLAKE3
identity and byte length, the attempt, optional `predecessor` and
`consumer_binding`, and the four producer ceilings:

```json
{
  "attempt": "attempt-1",
  "program": "p.ngraph", "program_identity": "<64 hex>", "program_bytes": 292,
  "inputs": "p.vectors", "inputs_identity": "<64 hex>", "inputs_bytes": 77,
  "fuel_interactions": 2097152, "max_input_bytes": 1048576,
  "max_retained_bytes": 67108864, "max_output_bytes": 65536
}
```

Unknown fields, including any snapshot or resume field, are refused.

## Bounds

| Bound | Owner | Profile ceiling |
| --- | --- | --- |
| Interaction fuel | Noble `--fuel` | 2,097,152 interactions |
| Accounted input bytes | Noble `--max-input-bytes` | 1 MiB |
| Accounted retained bytes | Noble `--max-retained-bytes` | 64 MiB |
| Report bytes, charged before rendering | Noble `--max-output-bytes` | 64 KiB, below the 128 KiB stdout bound |
| Agent slots, wires, worklist entries | Noble fixed capacities | 1,048,576 / 2,097,152 / 1,048,576 |
| Wall deadline, stdin/stdout/stderr, cancellation, teardown | existing process runner | 60 s, 4 KiB, 128 KiB |

An invocation may only lower the producer ceilings. Before dispatch the worker
adapter remeasures both artifacts (regular file, no link, exact length and
BLAKE3 identity), re-admits them through the closed vocabulary and validates the
host binding and limits. After the run it requires the report header to echo the
pinned producer revision, `ngraph-net-i64/3` contract, rule table, effective
limits and exact input byte count.

The byte ceilings are the producer's accounted ceilings, not process RSS bounds.
Producer counters (`interactions`, `peak_live_agents`, `retained_bytes_estimate`,
`retained_bytes_high_water`) are published as canonical decimal strings with their
own names; ceilings are never copied into measured fields.

## Results

Process disposition, evaluation outcome, output admission and publication stay
separate facts. The published document schema is `lattice.pure-inet-result.v1`:

| Evaluation | Operation state | Publication |
| --- | --- | --- |
| `complete` (exit 0, header plus one census-checked row, one canonical `i64:` output) | `Completed` | authoritative result, outcome `completed` |
| `exhausted` (one `E-NET-BUDGET-*` line, exit 5) | `Failed` | diagnostic, outcome `non-authoritative-diagnostic` |
| `invalid` (crossed binding, malformed or non-string output, wrong arity, census mismatch, producer reject or unsupported) | `Failed` | diagnostic |
| `partial` (header without a row) | `Failed` | diagnostic |
| `failed` (no report) | `Failed` | diagnostic |
| process timeout, cancellation, truncation or lost observation | `Unknown` | none |

A diagnostic is never a result: it has no `result`, its `authoritative` field is
false, and the run record has no output identity.

## Semantic count (#22)

Each complete document carries `semantic_interactions` version 1: the producer
counter as a decimal string, its unit (`inets-principal-pair-interaction`),
boundary and counter, and assurance `observed`. When the receipt names the
qualified theorems (`i64_normal_form_unique`, `i64_equal_interaction_count`,
`Certificate.accepted_exact_count` over `NobleFoundation.NetI64.system`) and the
exact proof-source identities, and the unit, census `2*ops+dups+eras`, basis
`census-checked` and per-row census equality all match, the document also
carries those bindings. A receipt that names no theorem stays observed only; a
receipt whose named binding differs fails admission.

`lattice::inet::count::compare` promotes a count to `checked` only for two fresh,
complete, admitted evaluations of one immutable computation (same program,
inputs, producer, executable, contract, profile and output contract; different
attempts and invocations) whose outputs and bridge-bound counts agree. A
different output is `OutputIntegrityFailure`; with equal outputs, a different
count is `InteractionCountIntegrityFailure`. `limit_consequence` then requires
fuel at or above a checked count `K` to complete with exactly `K` interactions
(success at exactly `K`); fuel exhaustion at or above `K` is an integrity
failure, fuel below `K` is expected exhaustion, and every other producer or
process bound remains an independent stop.

`checked` means census-checked recomputation consistency under model-only
theorems. It is not a refinement proof of Noble's Rust lowering, reducer or
read-back, and no proof was replayed by Lattice.

## Recovery

The existing durable owner records `Started` before dispatch and refuses a
second dispatch of the same invocation identity, so an unsettled or observed
attempt is never redispatched. `lattice::inet::lineage::admit_recomputation`
admits a fresh attempt only after the predecessor is terminal or explicitly
reconciled, within the attempt budget; the fresh invocation names its
`predecessor`, is planned and authorized again, and reconstructs the net from the
immutable artifacts. No heap, pointer graph, allocator or process snapshot is
resumed, and admitting a recomputation does not claim the predecessor never ran.
`admit_for_consumer` lets a consumer refuse an otherwise valid result whose
`consumer_binding` is no longer current; the consumer owns that binding.

## Verification

`tests/test_pure_inet_profile.rs` covers the closed vocabulary and the receipt,
count, recomputation, limit and lineage decisions against actual Noble receipts
in `tests/fixtures/pure-inet/`. `tests/test_pure_inet_worker.rs` runs the pinned
executable through the authored pack and the production route: an independent
Rust wrapping comparator over overflow, shared subexpressions and integers beyond
binary64 precision; producer fuel, retained, output and input exhaustion;
changed, crossed, linked, cyclic and unsupported artifacts; cancellation;
restart with a seeded `Started` attempt; interrupted publication; consumer
supersession; and the checked-count recomputation. The descendant-teardown case
uses a stand-in executable on the same adapter, because the Noble worker spawns
no descendants. Both targets run in the `pure-inet-profile` flake check.

## Non-claims

- The profile is sequential; no multicore, GPU or device reduction is qualified.
- Process bounds and a clear environment are not an OS sandbox.
- Producer byte ceilings are accounted ceilings, not RSS bounds; a 60 s
  deadline is enforced by the process runner, not by the producer.
- No live Mneme consumer, production deployment, default adoption, performance
  guarantee or universal confluence claim is made.
- A result's byte identity does not prove domain truth or Mneme correctness.
