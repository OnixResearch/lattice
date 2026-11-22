# VM execution profile, attempt binding and host model gateway

Lattice #10. An implementation attempt (`lattice.implementation-attempt-run.v3`)
can run inside a no-NIC Kiln guest VM. The guest holds no model credential.
Its model broker forwards every admitted request over vsock to a host process,
`lattice model-gateway serve`. That process holds the credential and asks Kiln's
effect authority before and after each upstream call.

This page defines four versioned records, C1–C4. Kiln consumes them only
through the `lattice` process boundary and the fixtures under
[`fixtures/vm-profile/`](../fixtures/vm-profile). It does not link this code.

Evidence for this revision is **Q0** (pure and fixture tests) and **Q1**
(in-process gateway and guest broker over real Unix sockets standing in for
the CH vsock endpoint). Model responses are scripted. Nothing here is
evidence of a real VM, real vsock, a live model, more than one host or a
production placement authority.

## Identities

Every identity uses one framing: BLAKE3 over

```
u64be(len(domain)) ‖ domain ‖ u64be(len(json)) ‖ json
```

It is rendered `b3:<64 lowercase hex>`. The domain is the record's schema
string. `json` is the compact serde encoding: fields in declaration order (the
order below), with absent optional fields omitted. Every record denies unknown
fields. [`vectors.json`](../fixtures/vm-profile/vectors.json) publishes the
canonical bytes and identities of the admitted fixtures.

## C1 `lattice.vm-execution-profile.v1`

C1 is authored in Nickel ([`profiles/vm-execution/`](../profiles/vm-execution)),
evaluated to JSON, then admitted. `contract.ncl` is structural only.
Lattice's admission (`src/vm_profile/core.rs`) is authoritative.

| Field | Admitted values |
|---|---|
| `schema` | `lattice.vm-execution-profile.v1` |
| `architecture` | `x86_64` or `aarch64` |
| `vmm` | `{ kind: "cloud-hypervisor", version }`, version non-empty without whitespace |
| `guest_manifest_schema` | `kiln.lattice-worker-guest.v1` |
| `guest_network` | `none` only |
| `gateway` | `{ exchange_vsock_port, model_vsock_port, host_peer_cid: 2, exchange_transport_schema: "lattice.workflow-vsock-transport-profile.v1" }`. The two ports differ, and neither is 0 or `u32::MAX` |
| `mobility` | non-empty, strictly ascending subset of `pinned`, `stopped_transfer`, `live_precopy`, `application_checkpoint` |
| `checkpoint` | `none` or `workflow-node-boundary-v1` |
| `deadline_authority` | `host` only |
| `resources` | `{ vcpus, memory_mib, workspace_mib }`, all non-zero |
| `devices` | strictly ascending subset of `virtio-blk-raw`, `vsock` that includes `vsock` |
| `non_claims` | at most 16 unique bounded strings, including the four in `REQUIRED_NON_CLAIMS` |

Admission refuses before any effect:

- `live_precopy` in `mobility` is refused (`live_precopy_unqualified`). This
  mirrors Kiln's `require_live_capability`.
- `application_checkpoint` requires a `checkpoint` other than `none`
  (`checkpoint_required`).
- A requested mode outside `mobility` is refused (`mobility_not_offered`).

Mobility maps to Kiln's `DrainDisposition` as follows: `pinned` → `Pinned`;
`stopped_transfer` and `live_precopy` → `Migrated`; `application_checkpoint`
→ `Checkpointed`.

## C2 `lattice.vm-attempt-binding.v1`

```
{ schema, profile: b3 identity of C1, guest_image, environment, envelope,
  attempt_id, fence, execution, workspace: { identity, generation },
  requested_mobility, deadline_unix_ms }
```

`guest_image`, `environment`, `execution` and `workspace.identity` are Kiln
content identities (64 lowercase hex). `envelope` is
`ImplementationAttemptEnvelope::identity()`. The binding names no placement
and no host endpoint, so a move keeps its identity. A binding with a placement
or endpoint field does not decode.

`admit_binding(binding, expected)` first admits the profile, then checks the
binding's own shape (`malformed_binding`). It then compares every value the
verifier knows independently, each with its own code: `crossed_profile`,
`crossed_guest_image`, `crossed_environment`, `crossed_envelope`,
`crossed_attempt`, `crossed_fence`, `crossed_execution`, `crossed_workspace`
and `crossed_deadline`. Last, it admits the requested mode. The guest knows the
profile, envelope, attempt and fence. Kiln and the host gateway know the rest.

### In the attempt request

A `lattice.implementation-attempt-run.v3` request may carry two optional
fields: `vm_binding` (C2) and `vm_profile` (the evaluated C1). Both are
per-attempt values bound by the exact request bytes, not by the profile
revision. When both are absent, they are absent from serialization, so the
request bytes, identity and behaviour are unchanged. When either is present,
validation requires:

- `vm_binding`, `vm_profile` and `omp.model_broker.forwarded` all present;
- a requested mobility other than `application_checkpoint`: the attempt route
  has no node-boundary checkpoint hooks, so it never restores from one;
- `network: false` declared explicitly;
- the binding admitted against the profile, `admitted_envelope`, the
  envelope's `attempt_id` and `fence`, and a requested mode the profile offers;
- `forwarded.binding` equal to the binding identity, and `forwarded.vsock_port`
  equal to `gateway.model_vsock_port`;
- no model credential in the request.

A broker profile with `forwarded` but no binding is refused. With a binding,
`implementation-attempt inspect` reports `isolation: vm-guest` and
`network: host-gateway-model-broker`. It always reports `checkpoint` as
`unavailable`, whatever checkpoint profile the VM execution profile declares.

## C3 `lattice.vm-effect-authority.v1`

Kiln serves this per host on a Unix socket with mode 0600. Each connection
carries one request. Frames are a 4-byte big-endian length followed by JSON, as
in the workflow exchange, bounded at 64 KiB. Messages are tagged by `kind`.

```
{ kind: "admit", binding, request_index: u32, model, route: "primary" }
  -> { kind: "admitted", placement_host, placement_generation: u64,
       consumed_requests: u32, remaining_requests: u32, deadline_unix_ms: u64 }
   | { kind: "refused", code: not_current | deadline | budget | revoked | unknown_binding }
{ kind: "settle", binding, request_index, outcome: completed | failed | timed_out | cancelled | uncertain,
  usage?: { input_tokens?, output_tokens? } }
  -> { kind: "recorded", consumed_requests: u32 } | { kind: "refused", code }
```

`consumed_requests` in `admitted` counts this binding's requests including the
admitted one. `remaining_requests` counts what remains after it. Kiln persists
the counter per binding and moves it with the placement, so a new host
continues the budget. The gateway checks each `admitted` reply:

- an `admitted.deadline_unix_ms` different from the binding's is refused as
  `deadline`, because a deadline is never extended;
- `consumed_requests == 0`, or `consumed_requests + remaining_requests` above
  the upstream ceiling, is refused as `budget`;
- an empty or control-character `placement_host` is refused as `not_current`.

A lost `admit` reply refuses the request and never contacts the upstream; the
authority may still count it consumed. A lost or refused `settle` leaves the
request `uncertain` on both sides. The gateway withholds the response and never
retries the upstream.

## C4 `lattice.model-broker-forwarded-upstream.v1`

The broker `Profile` gains an optional field,
`forwarded: { schema, vsock_port, binding }`, which is absent from serialization
when unset. When it is present, `credential` must be `null`, and the broker
connects AF_VSOCK to CID 2 at `vsock_port`. Each request uses one connection
with one frame in each direction:

```
request  { binding, request_index: u32, body: String }
response { request_index, status: u16, body } | { request_index, refused: code } | { request_index, uncertain: true }
```

The C4 refusal codes are the C3 codes plus `malformed`, which the gateway uses
for a body that is not JSON or names another model. Frame bounds derive from
`crate::http::MAX_REQUEST_BODY_BYTES` and `MAX_RESPONSE_BODY_BYTES`.

`request_index` strictly increases per binding and is never reused; it need
not be consecutive. Each broker run of an attempt serves at most
`max_requests` requests, so repair step `n` (from 1) numbers its requests from
`(n - 1) * max_requests` and a single-shot attempt from 0. The range is
derived from the retained step iteration, so a restarted controller
reproduces it. The gateway refuses a reused or lower index as `not_current`
before any authority or upstream call.

The guest broker keeps the direct route's settlement semantics: a delivered
response settles `Responded`, a guest-side timeout `TimedOut` and cancellation
`Cancelled`. A refusal, an `uncertain` reply, a lost or malformed reply, or a
reply for another index settles `TransportFailed`, and the worker receives the
broker's 403. Nothing is resent. A VM-bound broker connects to its gateway
before the worker starts. Without one, as on a host without guest vsock, the
attempt is blocked before spawn and never falls back to a host-process broker.

## Host gateway

```
lattice model-gateway serve --profile <C1.json> --binding <C2.json> \
  --upstream <lattice.model-gateway-upstream.v1.json> \
  --listen-uds <vsock socket>_<model_vsock_port> --vmm-pid <CH pid> \
  --authority-socket <C3 socket>
```

`--upstream` is `{ schema: "lattice.model-gateway-upstream.v1", endpoint,
model, credential?: env SecretRef, max_requests ≤ 256 }`. It holds the
host-only values the guest never sees. The process works as follows:

1. It admits the profile and binding, refuses to start at or after
   `deadline_unix_ms`, and resolves the env credential once.
2. It binds `--listen-uds` with mode 0600. This is the socket Cloud Hypervisor
   connects for guest-initiated vsock connections. It prints one
   `lattice.model-gateway-serve.v1` `listening` line.
3. It drops every connection whose `SO_PEERCRED` PID is not `--vmm-pid`, and
   every connection that closes without a complete frame, such as the guest's
   readiness probe.
4. For each frame it refuses, without calling the authority, when any of
   these hold:
   - the binding is crossed (`unknown_binding`);
   - the host clock is at or past the deadline (`deadline`);
   - the request index is not above the last one sent to the authority
     (`not_current`);
   - the local ceiling is reached (`budget`);
   - the body is malformed (`malformed`).
5. It sends `admit`, makes exactly one upstream call with the host credential,
   then sends `settle`. It returns the response only after `recorded`. A
   response that echoes the credential, or exceeds a bound, is withheld and
   settled `failed`.
6. It keeps serving refusals past the deadline until its owner stops it.

## Non-claims

- No real VM, real vsock, CH socket naming or Kiln C3 server was exercised.
- No live model endpoint or credential was used (OD-3).
- No cross-host move, production placement or fencing authority, or revocation
  provider is qualified (OD-2, Aspen).
- Kiln reimplements C1–C4 from the fixtures, under OD-1.
