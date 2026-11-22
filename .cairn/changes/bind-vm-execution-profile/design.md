# Design: VM execution profile, attempt binding and host model gateway

## Context

Baseline: Lattice `main` `5a8e1d51`. The relocation plan fixes the contracts C1–C4 and decisions D2 (split broker), D3 (identities without placement), D4 (host-authoritative deadlines) and D6 (no v1 Kiln wire change).

What exists:

- `implementation-attempt run` validates `lattice.implementation-attempt-run.v3`. It admits only `network: false` when declared and checks expiry against the local clock (`src/implementation_attempt/cli.rs`).
- The attempt-local broker (`src/candidate_worker/broker/`) serves `POST /v1/chat/completions` on a private Unix socket. It resolves the credential on the host and passes every request through a `ModelRequestGate` (`DirectRoute` or `OwnerGate`), settling each admitted call exactly once with `Upstream::{Responded, TimedOut, TransportFailed, Cancelled}`.
- Workflow-exchange identities use length-prefixed, domain-separated BLAKE3 (`src/workflow_exchange/core.rs`).

## Decision 1: pure C1/C2 core with typed refusals

`src/vm_profile/` holds only records, admission and identity. Values a profile may carry but this revision refuses (another guest network, an unknown device, `live_precopy`) remain representable, so admission names them with a code instead of a parse error. Unknown fields still fail to parse. Mobility and devices must be strictly ascending, which gives each set one canonical encoding and therefore one identity. `admit_binding` takes an `Expected` with optional fields: the guest checks profile, envelope, attempt and fence; Kiln and the gateway can check the rest. The Nickel contract is structural only, and Rust admission is authoritative.

## Decision 2: the request carries C2 and C1

The plan names `vm_binding: Option<C2>`. Validating "requested mobility is in the profile" and projecting the profile's `checkpoint` in `inspect` both need the profile's content, not only its identity. So the request also carries `vm_profile: Option<C1>`. The two fields and `omp.model_broker.forwarded` are present together or absent together. Neither is part of the profile revision, so a bound request keeps the revision of its unbound twin.

## Decision 3: forwarding is a broker profile field, not a new broker

`Profile.forwarded` selects the forwarded upstream inside the existing `wire::Handler`. Bounds, model check, body canonicalization, gate admission and settlement stay as they are, so the guest keeps the direct route's semantics. The upstream call becomes one C4 frame on a fresh AF_VSOCK connection. A reply maps to `Upstream` as follows:

- a delivered response maps to `Responded`;
- a guest-side deadline maps to `TimedOut`;
- broker stop maps to `Cancelled`;
- every refusal, `uncertain`, lost, malformed or crossed reply maps to `TransportFailed`.

The vsock fd is wrapped as a `UnixStream` only for reads, writes, timeouts and shutdown, which are family-independent; `unsafe` stays forbidden. Before the worker spawns, `host::with_route` connects once to the gateway and drops the connection. Without a gateway the attempt is blocked before spawn and never falls back to a host route.

## Decision 4: the gateway is a `ModelRequestGate` over C3

`KilnAuthorityGate` implements `ModelRequestGate`: `prepare(index)` then `admit(checks)` sends `Admit`; `settle` sends `Settle`. The shell uses its `settle_outcome` with the richer C3 outcome (`completed` for 2xx, `failed` for other statuses and withheld responses, `timed_out`, `cancelled`, `uncertain` for transport failures). Pure checks in `model_gateway/core.rs` decide, before any authority call:

- a crossed binding is refused as `unknown_binding`;
- a request at or past the host deadline is refused as `deadline`;
- a non-increasing index is refused as `not_current`;
- a request beyond the local ceiling is refused as `budget`;
- a malformed body or another model is refused as `malformed`.

The checks also reject an authority reply that moves the deadline or reports consumption beyond the ceiling. A gateway restarted on a new host trusts the carried `consumed_requests`, not a local counter, so the budget continues across a move.

## Decision 5: host deadline and peer identity

The gateway compares each request with the host clock, and caps the upstream timeout at what remains. It refuses to start at or past the deadline. Past the deadline it keeps answering `deadline` until its owner stops it, so a guest resumed after a long pause learns why. It drops any connection whose `SO_PEERCRED` PID is not `--vmm-pid`, as Kiln's peer-PID rule does.

## Contract gaps resolved here (reported to the integration owner)

- **C3 framing:** the plan gives field lists, not an encoding. Frames are internally tagged with `kind` (`admit`, `settle`, `admitted`, `recorded`, `refused`) and carry no `schema` field. The version is pinned by the C1/C2 identities and the socket handover.
- **C3 counters:** `consumed_requests` includes the admitted request; `remaining_requests` counts what remains after it.
- **C4 refusal codes:** the C3 codes plus `malformed`, for gateway-local body refusals.
- **Gateway upstream:** the plan's argv has no endpoint, model or credential reference, so `--upstream` names a new `lattice.model-gateway-upstream.v1` record with an env-only credential.
- **C4 response content type:** C4 carries no headers. The guest labels a body `text/event-stream` only when it is not JSON and has `data:` lines.

## Evidence classes

Q0 covers the pure core and the fixtures. Q1 covers the in-process gateway shell and the guest `Handler` over real Unix sockets standing in for the CH vsock endpoint, with a scripted C3 authority and a scripted OpenAI-compatible upstream. No VM, real vsock, live model, second host or production authority was exercised.
