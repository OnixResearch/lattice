# Design: Charge observed model requests from complete broker evidence

## Context

Baseline: Lattice `b6fefb10`. The OMP attempt broker (`candidate_worker::broker`) relays every worker model request and numbers it with a request index (`first_request_index + requests`, `0` for a single shot, `(n - 1) * max_requests` for repair step `n`). A served binding passes each request through `OwnerGate`, whose serving session writes `model-requests/<hex>.{intent,record}.json` through `binding::requests::RequestFiles`; every other binding used `DirectRoute`, which wrote nothing. `requests::coverage` summarized served evidence and reported `unknown` otherwise. `repair::single_shot_draw` and the operator `OmpPort` charged the broker ceiling with `unknown` provenance unconditionally, and `Loop::start` charged `max(reported, reservation)`.

## Decisions

### Decision: brokered evidence for every attempt without a serving declaration

**Choice:** a `RecordedRoute` gate replaces `DirectRoute` for attempts. It keeps the broker's own checks, writes a `lattice.brokered-model-request-dispatch.v1` intent through `RequestFiles` (create-new, fsync of file and directory) before upstream contact, refusing the request if that write fails, and a `lattice.brokered-model-request-observation.v1` record after settlement, failing the broker if that write fails. Records carry binding identity, attempt, fence, route, request id and request index, an outcome (`completed` or `uncertain-*`) and provider-reported counters when both are present. The request id is the domain-separated BLAKE3 (`lattice brokered model request v1`) of the binding identity and request index. `ModelRequestGate::admit` now receives the broker's request index. `RequestFiles::new` creates `model-requests/` and syncs the attempt directory before the broker starts, so a retained empty directory proves no request reached upstream and a missing one is no evidence.

**Rationale:** the broker is the only egress, so the same write-once, intent-before-upstream discipline as served evidence makes its count exact. Keying by the broker's index, rather than a per-run ordinal, keeps request ids unique across repair steps and makes per-step attribution a pure range filter. Served evidence keeps its schema; the two never mix (served bindings parse only served files, others only brokered files).

### Decision: `brokered_unattributed` coverage

**Choice:** `ModelRequestCoverage::BrokeredUnattributed(brokered::Summary)` with `is_count_complete`, dispatched, unconfirmed, uncertain and unknown-usage counts, known tokens and every entry in index order; no service, deployment, grant or replica field. `unknown` stays for a binding without a serving declaration and without `model-requests/`. `brokered::summarize` refuses schema, crossed owner, undeclared route, mismatched request id, conflicting duplicates, a record without its intent and a served binding (`Conflict::Served`). `ModelRequestCoverage::observed_requests(indices)` returns the exact count, or `None` for an intent without a record, an `uncertain` request, `unknown`, `no_model`, or served evidence asked for an index range.

**Rationale:** callers must tell measured-but-unattributed apart from unobserved; reusing `unknown` would hide the measurement and reusing `served` would invent attribution.

### Decision: charge exactly, retain, replay

**Choice:** `repair::step::Report` gains `observed_model_requests`; `Loop::start` charges it exactly with `host_measured` provenance and otherwise keeps `max(reported, reservation)`. `OmpPort` creates `RequestFiles` before each launch, gives the launch a recorded gate, and after the launch reports `coverage.observed_requests([(n - 1) * max, n * max))`. For a single shot, `WorkerStep` persists the coverage, then `SingleShotDraw::retain` writes `lineage-draw.json` write-once before the result is published; recovery returns the retained draw after checking its binding, unaccounted dimensions and charge shape, and an attempt without one reports the ceiling.

**Rationale:** the step and single-shot charges are then a function of write-once evidence taken before the result, and replays read retained charges rather than evidence, so lost or tampered evidence can never lower a reported draw. An attempt from before this change has no retained draw and keeps the ceiling it always reported.

## Risks / Trade-offs

- A step or single shot now draws less than one broker ceiling when fewer requests ran; Kiln `90a9fb64` does not read `budget_lineage.usage[]`, so it is unaffected, and a future consumer must treat `charged` as authoritative.
- Single-shot tool invocations stay unaccounted and repair-step tool invocations stay at their reservation; only model requests are observed.
- Evidence and summarization now cover up to 2,304 requests (4,608 files) per attempt: nine broker runs of 256.
