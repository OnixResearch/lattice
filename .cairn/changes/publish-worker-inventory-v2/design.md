# Design: Worker inventory v2

## Context

Baseline: Lattice `1bdacb354ec183e8485e3ed702150d0323bd8c25`. Consumer: Kiln `4371dec083d446106bcc03ef01d6fe2d41850164`.

What exists:

- `crates/lattice-core/src/execution_binding/projection/inventory.rs` projects `lattice.worker-profile-facts.v1` from owner-supplied `Offered` facts. Each fact is `declared`, `qualified`, `unavailable` or `unknown`, and `qualified` needs a canonical `qualification_ref`. Two owners feed it: `src/agent_node/runtime/profile.rs` for configured Pi, Clankers and Animus routes, and `src/implementation_attempt/cli/inspect/mod.rs` for the OMP attempt worker.
- `projection/retained.rs` projects `lattice.worker-instance-observation.v1` from retained attempt artifacts read by the shell: binding, intent, refusal, cancel request, worker outputs and the published terminal result. Host reachability and allocation liveness are `OwnedUnknown { unknown, kiln#49 }`.
- `inspect` prints `lattice.worker-inventory.v1 { profile, instance, model_requests }`.
- Kiln's reader denies unknown fields at every level, maps a profile revision to a definition id, and lists `task_families`, `tools`, `revisions` and `qualifications` as unknown. No Lattice state maps to idle or healthy.
- The worker configuration's typed authoring contract is `contracts/worker-execution-binding.ncl`, decoded by `src/execution_binding/format.rs`. The implementation-attempt request (`lattice.implementation-attempt-run.v3`) is its exported per-attempt form. Its `worker_profile_identity` is BLAKE3 over a JSON tuple of program, sandbox, OMP profile, paths, output bound, `allow_unchanged` and `worker_compute`.

Kiln asks for a definition identity, declared task families and tools, and a liveness or idle signal. It also asks that `attempt_id`, `fence` and `work_id` stay unchanged.

## Goals and non-goals

Goal: publish those facts, plus bounded wait reasons and explicit #25 slots, as one clean v2 schema, without inventing observations Lattice does not make.

Non-goals: a heartbeat or reusable runtime, #25's lifecycle, task matching, qualification evidence and the Kiln reader.

## Decision 1: clean v2 cutover at every level

The envelope, profile facts and instance observation all move to `.v2`. Kiln's v1 DTOs deny unknown fields, so an additive v1 change would break them silently. Lattice keeps no v1 constant, emitter or alias. Configured-route facts use the same type and therefore also become v2.

Rejected: a v1/v2 dual emitter. It would need a selector flag and two maintained shapes for a consumer that will be updated anyway.

## Decision 2: a definition identity from owner, basis and name

```text
definition = { identity: "blake3:<hex>", name: <name>, basis: "declared" | "owner_default" }
identity   = content_identity::worker_definition { owner, basis, name }   // domain lattice-worker-definition-v1
```

The name is the operator's declared `worker_capability.definition` or the owner's default: `omp-attempt-worker` for attempts and the target label for configured routes. Hashing the basis keeps a declared name that equals a default name distinct from the default. The revision, adapter, executable, capability sets and routes are deliberately not inputs, so a new revision of the same definition keeps its identity. The identity uses the shared `content_identity_core` framing (tagged strings) like the other Lattice roles, with a pinned test vector.

Every instance carries `definition_identity`, which the shell passes from the projected profile. The admitted binding is unchanged, so `binding_identity`, `attempt_id`, `fence` and `work_id` stay byte-identical.

Rejected: hashing the adapter into the definition. A Pi target moved from a command route to a direct route is the same agent definition at a new revision.

## Decision 3: an optional capability declaration in the worker configuration

```text
WorkerCapability (lattice.worker-capability.v1) {
  schema, definition: Name,
  task_families?: { kind = "declared", names: [Name; 1..=32] } | { kind = "unavailable" },
  tools?:         { kind = "declared", names: [Name; 1..=32] } | { kind = "unavailable" },
}
Name = ^[a-z0-9][a-z0-9._:-]*$, at most 64 bytes
```

The contract field and the request field are both optional. The pure core (`execution_binding::capability`) validates a declaration and returns its canonical form, with names sorted. Three places call it before any effect: the strict Nickel decoder (`format::load_configuration`), `Request::validate_binding` and the profile projection. The Nickel predicate refuses the same shapes at evaluation time.

The profile revision frames the canonical declaration only when one is present. `worker_profile_identity` hashes the original 8-tuple when it is absent and a 9-tuple with the canonical declaration otherwise, so every existing request keeps its revision. Name order does not change the revision.

The name grammar excludes `/`, whitespace, uppercase, `=` and quotes, so a name cannot carry a path, command line, prompt, URL or typical secret syntax. The 32-name and 64-byte bounds cap a profile's capability text at about 4 KiB.

Facts:

| Declaration | `task_families` / `tools` fact |
|---|---|
| no `worker_capability` | `unknown`, `value: null` |
| set absent | `unknown`, `value: null` |
| `declared` | `declared` (or `qualified` with bound evidence), sorted names |
| `unavailable` | `unavailable`, `value: null` |

An empty declared set is refused rather than published, because it would be indistinguishable from "unavailable" and from undeclared data rendered as `[]`.

Rejected: per-name support levels. Nothing qualifies an individual name today (Animus #18), and per-profile qualification already promotes the whole fact.

## Decision 4: single-shot liveness from retained state

```text
liveness = { runtime: "single_shot", state: not_started | exited | unknown,
             unknown_cause: null | unobserved | stale,
             retained_generation: u64, observed_at_epoch_ms: i64 }
```

| Lifecycle | state | unknown_cause |
|---|---|---|
| `admitted_not_dispatched`, `binding_rejected_before_dispatch` | `not_started` | — |
| `terminal_observed` | `exited` | — |
| `dispatch_recorded_outcome_unknown`, observed at or before the request's `expires_at_ms` | `unknown` | `unobserved` |
| `dispatch_recorded_outcome_unknown`, observed after `expires_at_ms` | `unknown` | `stale` |

The shell supplies the observation time and the admitted window (`expires_at_ms`), and the pure core compares them. `retained_generation` counts the retained write-once artifacts plus the terminal result. Artifacts are never removed, so the generation never decreases for one attempt. Lattice owns no heartbeat or reusable runtime, so `idle` and `live` are not in the vocabulary. A future reusable runtime needs a heartbeat owner and a new schema, not a reinterpretation of this one.

Rejected: using the mtime of `raw-events.jsonl` as a heartbeat. That file is written once, after the worker exits.

## Decision 5: closed wait reasons

```text
wait_reason = { code, subject: { kind: "fact", field } | { kind: "route", name }, owner }
```

| Code | Where | Subject | Owner |
|---|---|---|---|
| `capability_undeclared` | profile | `task_families` / `tools` | `configuration` |
| `qualification_unbound` | profile | `qualification_ref` | `animus#18` |
| `revision_selection_unpublished` | profile | `revisions` | `lattice#25` |
| `serving_port_absent` | both | `serving_readiness` | `kiln#49` |
| `serving_snapshot_unavailable`, `serving_port_issue`, `serving_snapshot_not_current`, `serving_not_ready` | both | route name | `animus#17` |
| `awaiting_dispatch`, `admission_expired` | instance | `lifecycle` | `kiln#45` |
| `dispatch_outcome_unknown`, `cancellation_unobserved` | instance | `lifecycle` / `cancellation` | `lattice#6` |
| `liveness_unobserved`, `liveness_stale` | instance | `liveness` | `kiln#49` |
| `host_reachability_unknown`, `allocation_liveness_unknown` | instance | that fact | `kiln#49` |

Each served route gives at most one reason. A snapshot that is not current gives `serving_snapshot_not_current` and never a readiness claim. A current snapshot whose service is not `ready` gives `serving_not_ready`. Terminal and rejected instances are not waiting and carry none. Reasons are sorted by `(code, subject)` through derived `Ord` and deduplicated. More than 32 reasons is a typed defect rather than truncation. By construction a profile has at most 4 + 8 reasons and an instance at most 5 + 8 (one per declared route, at most 8 routes).

Disclosure: the subject vocabulary is closed. A route subject is a name already printed in the same document's `model_routes`, so a reason discloses nothing the inventory does not already show. Service ids, endpoints, credential and grant references, backend handles, paths, prompts and commands never enter a reason.

## Decision 6: #25 slots as owned unknowns

`revisions = { desired, prepared, qualified, selected_for_new_sessions, pinned }`, where each slot is the existing `OwnedUnknown { support: unknown, owner: "lattice#25" }`. This reuses the established "owner has not published" shape. An empty or absent value would be ambiguous.

## Pure core and shell

| Pure (`lattice-core`) | Shell (`lattice`) |
|---|---|
| capability validation and canonical form; definition identity; profile facts with capability, definition, slots and reasons; instance liveness and reasons | reading the request and retained artifacts; the observation clock; passing `expires_at_ms` and the profile's definition identity; printing |

No new port, clock read or file read enters the core.

## Captures for Kiln

`capture_worker_inventory_v2` tests in `src/implementation_attempt/cli/inspect/tests.rs` and `src/implementation_attempt/cli/binding/tests/serving.rs` mirror Kiln's capture patch. They write to `$LATTICE_INVENTORY_CAPTURE_DIR` only when it is set: terminal failed, admitted not dispatched, dispatched outcome unknown, profile without instance, `agent-a-2`, served ready and served unavailable, plus a declared-capability attempt. `observed_at_epoch_ms` comes from the host clock and is the only nondeterministic field. The README in the capture directory records it.

## Tests

- Core (`tests/test_execution_binding_inventory.rs`): undeclared to unknown; declared sorted; unavailable; malformed declarations by defect; bounds at 32/33 names and 64/65 bytes; definition stable across revisions and distinct across bases; slots unknown; liveness for every lifecycle, including unobserved and stale; reasons sorted, deduplicated and drawn from published route names; no `idle`, `live`, `healthy` or `running` text.
- Identity (`tests/test_execution_binding_identity.rs`): pinned `lattice-worker-definition-v1` vector.
- Contract (`tests/test_execution_binding_contracts.rs`): a positive declared fixture; negative fixtures for each malformed shape in Nickel; a bypass refused by the strict decoder.
- Shell (`src/implementation_attempt/cli/tests.rs`, `cli/inspect/tests.rs`): revision unchanged without a declaration and changed with one; order-insensitive identity; inspect refuses a malformed declaration; no leakage of paths, prompt, endpoint or credential; v2 schema names only.
- Configured routes (`tests/test_agent_profile_bindings.rs`): owner-default definitions per target, capability unknown.

## Risks and claim boundaries

- Breaking for Kiln's v1 reader until the separate Kiln update lands. That is intended: the reader fails closed with `lattice_inventory_unsupported_schema`.
- Wait-reason owners are routing hints, not authority. A reason is never a reservation, a claim or a dispatch decision.
- The capture documents are synthetic fixture outputs. No live worker, provider, host or serving owner is observed.
- The name grammar reduces, but cannot eliminate, the chance that an operator writes something sensitive into a name. Names are operator-authored configuration, not worker output.
