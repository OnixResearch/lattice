# Projected role-protocol execution

Lattice executes one frozen Choregraph role-projection cohort. The cohort source is Choregraph revision `0b2000edaf86d5a7a8601246afbc0d526f4a2a07`.

## Intake

The intake adapter receives the Choregraph global protocol, complete ordered local set, and projection bundle. It checks these items:

- exact global, local, local-set, schema-set, proof, and receipt identities;
- exact packed Preserves bytes for each supplied DTO;
- the frozen Choregraph and Trellis revisions;
- one local artifact for each declared role; and
- normalized role order.

The adapter maps accepted producer data into Lattice-owned types. It does not compile or project a second protocol.

## Placement

A session plan contains one placement for each role. A native placement binds one runtime route and adapter cohort. An external placement also binds its contract, schemas, destination policy, credential reference, limits, and adapter cohort.

A missing, extra, reordered, mixed-kind, or mismatched placement fails before session persistence. Placement data does not grant authority.

## Published runtime profile

r[impl lattice.role_protocol.runtime_profile]

`profiles/role-protocol-runtime-v1.ncl` defines the reviewed consumer profile. Its canonical JSON and BLAKE3 sidecar are under `profiles/generated/`.

The profile binds runtime revision `cbb5db0e4844b466dcd1877218eb1f00a2a7caf2`. It also binds the selected source manifest, package, binary, command prefix, state backend, and authority contract.

The Nix profile check verifies that manifest against the separately pinned published runtime source, not the evolving checkout. Current protocol CLI boundaries and effect-authority behavior are checked independently.

The profile supports create, inspect, recover, retry, abandon, and replay. It describes an operator-invoked one-shot service that does not start at boot.

The CLI does not dispatch protocol effects. It uses unavailable authority and empty adapter registries. Effect dispatch remains a Rust library surface for an authorized product adapter.

OnixOS can use this profile for compatibility checks and dry-run service planning. OnixOS still owns the machine, service user, paths, package derivation, policy references, and activation decision.

A future daemon or dispatching CLI requires a new profile identity and separate review. Consumers must fail when requested behavior exceeds this profile.

## Durable order

A dispatch uses this order:

1. Plan one exact local action.
2. Persist the `Prepared` attempt and all canonical envelopes.
3. Evaluate the current Choregraph effect-authority boundary for each envelope.
4. Persist the successful authority-attempt references.
5. Persist `DispatchedUnknown` before the adapter starts.
6. Start the selected bounded adapter.
7. Admit an exact receiver observation.
8. Commit linked sender and receiver cursors.

A crash after step 5 leaves an unknown outcome. Lattice does not resend automatically. A terminal adapter rejection can fail the attempt only when no earlier message in that action started.

## Envelopes and choices

Each envelope binds the protocol, complete local set, session, source and destination instances, local artifacts, step, message, attempt, label, payload tag, optional value, payload reference, schema set, adapter cohort, and authority request.

Choices use exact labels. Lattice does not convert a label set into a Boolean. A stale, unknown, duplicated, reordered, wrong-role, wrong-step, wrong-value, or cross-session envelope cannot advance a cursor.

## Ownership

An owned value moves only after the exact receiver observation commits. The prior owner cannot send that value again until a later admitted transfer returns ownership.

This local commit does not prove remote byte uniqueness or remote durable storage.

## Recovery and replay

The typed operator surface supports create, inspect, recover, retry, abandon, and replay requests. Retry requires explicit duplicate-risk acknowledgement. Abandon requires explicit unknown-outcome acknowledgement. A retry receives a new action and attempt identity.

Replay reads the persisted plan and events. It recomputes pure state only. It does not load fresh authority or call an adapter.

## Safe evidence

Inspection reports bounded identities, status, counts, attempts, outcomes, issue codes, and the claim boundary. It omits payload bytes, commands, credentials, endpoint secrets, source text, and host paths.

The evidence reports local artifact admission, cursor decisions, durable attempt linkage, and bounded observations. It does not claim transport confidentiality, endpoint identity truth, remote durability, external-role correctness, exactly-once delivery, global progress, fault deadlock freedom, production readiness, or release eligibility.
