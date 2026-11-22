# VM Execution Profile Specification

## ADDED Requirements

### Requirement: VM execution profile admission
r[lattice.vm_profile.admission] Lattice MUST admit a `lattice.vm-execution-profile.v1` record only when it has the exact schema and a Cloud Hypervisor VMM with a bounded version, names the `kiln.lattice-worker-guest.v1` manifest schema, has a guest network of `none`, has distinct gateway vsock ports with host CID 2 and the `lattice.workflow-vsock-transport-profile.v1` exchange transport, offers a non-empty strictly ascending set of mobility modes, uses the `host` deadline authority, has non-zero resources, has a strictly ascending device set within `virtio-blk-raw` and `vsock` that includes `vsock`, and states the required non-claims. Every other record MUST be refused before any effect with a typed refusal code. Unknown fields MUST fail to decode.

#### Scenario: Unqualified and inconsistent mobility
r[lattice.vm_profile.admission.scenario.mobility]
- GIVEN a profile offering `live_precopy`, a profile offering `application_checkpoint` with checkpoint `none`, or a requested mode the profile does not offer
- WHEN admission evaluates it
- THEN it MUST be refused as `live_precopy_unqualified`, `checkpoint_required` or `mobility_not_offered`
- AND an admitted mode MUST map to Kiln's drain disposition: `pinned` to `pinned`, `stopped_transfer` and `live_precopy` to `migrated`, `application_checkpoint` to `checkpointed`

#### Scenario: Nickel-authored profiles
r[lattice.vm_profile.admission.scenario.nickel]
- GIVEN the positive and negative Nickel profiles under `profiles/vm-execution/`
- WHEN each is evaluated and admitted
- THEN the positive profile MUST evaluate to the published `admitted.json`, and every negative profile MUST fail its Nickel contract or Lattice admission with its named code

### Requirement: VM attempt binding
r[lattice.vm_profile.binding] Lattice MUST admit a `lattice.vm-attempt-binding.v1` record only when its profile is admitted and its identity equals the binding's `profile`, its content identities, attempt, fence and deadline are well formed, its requested mode is offered, and every value the verifier knows independently (envelope, attempt, fence, guest image, environment, execution, workspace and deadline) is equal. The binding MUST NOT contain a placement or a host endpoint.

#### Scenario: Crossed fields
r[lattice.vm_profile.binding.scenario.crossed]
- GIVEN a binding that differs from the verifier's expectation in exactly one field
- WHEN it is admitted
- THEN it MUST be refused before any effect with that field's code: `crossed_profile`, `crossed_guest_image`, `crossed_environment`, `crossed_envelope`, `crossed_attempt`, `crossed_fence`, `crossed_execution`, `crossed_workspace`, `crossed_deadline` or `mobility_not_offered`

### Requirement: Framed identities
r[lattice.vm_profile.identity] C1 and C2 identities MUST be BLAKE3 over `u64be(len(domain)) || domain || u64be(len(json)) || json`, rendered `b3:<64 lowercase hex>`. The domain MUST be the schema string, and `json` the compact encoding in declaration order with absent optional fields omitted. The published fixtures MUST reproduce their published canonical bytes and identities.

### Requirement: VM-bound implementation attempt requests
r[lattice.vm_profile.request] A `lattice.implementation-attempt-run.v3` request MAY carry `vm_binding` and `vm_profile`. When either is present, both and a forwarding broker profile MUST be present, `network` MUST be declared `false`, the binding MUST be admitted against the profile, `admitted_envelope` and the envelope's attempt and fence, the forwarding profile MUST name exactly this binding identity and the profile's model vsock port, and the request MUST carry no model credential. A forwarding broker profile without a binding MUST be refused.

#### Scenario: Absent binding
r[lattice.vm_profile.request.scenario.absent]
- GIVEN a request without `vm_binding`, `vm_profile` and broker `forwarded`
- WHEN it is serialized, identified and validated
- THEN none of these fields MUST be serialized, and its bytes, identities and behaviour MUST be unchanged
- AND a bound request MUST keep the profile revision of the same request without its binding

#### Scenario: Crossed or incomplete binding
r[lattice.vm_profile.request.scenario.crossed]
- GIVEN a bound request with a missing or undeclared network, a crossed envelope, attempt, fence or profile, an unoffered mode, a guest NIC, a missing binding, profile or forwarding profile, a forwarding profile for another binding or port, or a credential
- WHEN it is validated
- THEN it MUST be refused before any binding, intent or worker effect

#### Scenario: Inspect facts
r[lattice.vm_profile.request.scenario.inspect]
- GIVEN a bound request
- WHEN `implementation-attempt inspect` projects its profile facts
- THEN `isolation` MUST be `vm-guest`, `network` MUST be `host-gateway-model-broker` and `checkpoint` MUST be the profile's checkpoint profile, or `unavailable` for `none`; an unbound request MUST keep its existing facts

### Requirement: Forwarded model broker
r[lattice.vm_profile.forwarded_upstream] A broker profile MAY carry `forwarded: { schema: "lattice.model-broker-forwarded-upstream.v1", vsock_port, binding }`. A forwarding broker MUST send each admitted request as one `{ binding, request_index, body }` frame on a new AF_VSOCK connection to CID 2 at `vsock_port`, and MUST accept only a reply for that index: `{ status, body }`, `{ refused: code }` or `{ uncertain: true }`. It MUST NOT resend any request.

#### Scenario: No credential in the guest
r[lattice.vm_profile.forwarded_upstream.scenario.credential]
- GIVEN a forwarding broker profile
- WHEN it carries a credential reference, an unsupported schema or an invalid port
- THEN it MUST be refused, and a forwarding broker MUST never resolve a credential

#### Scenario: Settlement semantics
r[lattice.vm_profile.forwarded_upstream.scenario.settlement]
- GIVEN an admitted forwarded call
- WHEN the gateway responds, refuses, reports `uncertain`, answers another index, closes without a reply, or does not reply before the deadline or broker stop
- THEN the gate MUST settle `Responded`, `TransportFailed`, `TransportFailed`, `TransportFailed`, `TransportFailed`, `TimedOut` or `Cancelled` respectively, and only `Responded` MUST reach the worker

#### Scenario: No host-process fallback
r[lattice.vm_profile.forwarded_upstream.scenario.no_fallback]
- GIVEN a forwarding broker where no host gateway accepts a vsock connection
- WHEN the attempt's broker route is prepared
- THEN it MUST fail before the worker spawns, and MUST NOT fall back to a host-process broker or credential

#### Scenario: Request indices across broker runs
r[lattice.vm_profile.forwarded_upstream.scenario.request_index]
- GIVEN an attempt whose worker runs several broker incarnations, such as the steps of a repair policy, against one host gateway
- WHEN each incarnation forwards its requests
- THEN step `n` (from 1) MUST number them from `(n - 1) * max_requests`, so indices strictly increase across the attempt, are derived from the retained step iteration alone and are never reused; a single-shot attempt MUST start at 0
- AND the gateway MUST refuse a reused or lower index as `not_current` without an authority or upstream call

### Requirement: Kiln effect authority
r[lattice.vm_profile.effect_authority] The gateway MUST speak `lattice.vm-effect-authority.v1` to a Unix socket: one length-prefixed JSON frame each way per connection. `admit { binding, request_index, model, route }` is answered by `admitted { placement_host, placement_generation, consumed_requests, remaining_requests, deadline_unix_ms }` or `refused { code }`, with code `not_current`, `deadline`, `budget`, `revoked` or `unknown_binding`. `settle { binding, request_index, outcome, usage? }` is answered by `recorded { consumed_requests }` or `refused { code }`.

### Requirement: Host model gateway
r[lattice.vm_profile.gateway] `lattice model-gateway serve` MUST admit its profile and binding before listening, hold the only model credential, and pass each forwarded request through the local checks, then C3 `Admit`, then exactly one upstream call, then C3 `Settle`, through `KilnAuthorityGate` as a `ModelRequestGate`. It MUST return a response only after `recorded`.

#### Scenario: Authority refusals
r[lattice.vm_profile.gateway.scenario.authority]
- GIVEN an authority that refuses with `not_current`, `deadline`, `budget`, `revoked` or `unknown_binding`, or whose `Admit` reply is lost
- WHEN a request is forwarded
- THEN the guest MUST receive that refusal, or `not_current` for a lost reply, and the upstream MUST NOT be called

#### Scenario: Host-authoritative deadline
r[lattice.vm_profile.gateway.scenario.deadline]
- GIVEN a host clock at or past the binding's `deadline_unix_ms`, or an authority admission naming another deadline
- WHEN a request is forwarded
- THEN it MUST be refused as `deadline` without upstream contact, and without an authority call in the first case

#### Scenario: Deadline reached during admission
r[lattice.vm_profile.gateway.scenario.admit_latency]
- GIVEN an authority whose `Admit` reply arrives after the host clock reached the binding deadline
- WHEN the gateway prepares the upstream call
- THEN it MUST reread the host clock after `Admit`, MUST NOT call the upstream, MUST settle the request `cancelled` and MUST refuse it as `deadline`, or report it `uncertain` when that settlement is lost

#### Scenario: Lost settlement
r[lattice.vm_profile.gateway.scenario.lost_settle]
- GIVEN an admitted call whose `Settle` reply is lost
- WHEN the upstream call completed
- THEN the guest MUST receive `uncertain`, the response MUST be withheld, and a replay of the same index MUST be refused without a second upstream call

#### Scenario: Budget continuity across hosts
r[lattice.vm_profile.gateway.scenario.budget_continuity]
- GIVEN a new gateway incarnation whose authority carries the consumption of a previous host
- WHEN requests are forwarded
- THEN only the carried remainder MUST reach the upstream, and an authority reporting consumption beyond the ceiling MUST be refused as `budget`

#### Scenario: VMM peer only
r[lattice.vm_profile.gateway.scenario.peer]
- GIVEN a connection to the gateway socket from a process other than `--vmm-pid`
- WHEN it sends a frame
- THEN it MUST be dropped without a reply, an authority call or an upstream call
