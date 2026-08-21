# Admitted execution backend orchestration

Lattice selects and persists one exact operation route before an effect. Animus supplies pure backend-selection advice. It does not grant Lattice authority.

## Source cohort

Lattice pins the Animus execution-backend planner contract at revision `54b2fdaf57ac9bbc34e6c73f75d803d625f84ec0`.

Typed positive and negative Nickel fixtures freeze the reviewed projection. Lattice does not import Animus host adapters or transfer product authority.

## Ownership

| Owner | Responsibility |
|---|---|
| Animus planner | Select the weakest compatible backend from explicit requirements, policy, and candidate facts. |
| Lattice operation core | Validate requirements, map planner values, and revalidate the selected route against product-owned facts. |
| Lattice shell | Persist attempts, enforce authority, resolve credential references, dispatch the exact route, and record observations. |
| Lattice store | Commit the selected route and attempt before dispatch. Preserve unknown outcomes without automatic substitution. |

A planner selection cannot override a denied runtime, adapter, effect policy, authority policy, credential reference, budget, persistence, or recovery decision.

## Requirement profile

Each operation declares one value for each execution dimension:

- format;
- filesystem semantics;
- network posture;
- process model;
- terminal need;
- browser or device need;
- persistence need; and
- requested isolation claim.

The profile also declares positive memory, storage, CPU-time, and wall-time bounds. Unknown enum values and non-positive bounds fail before route selection.

## Durable route

The invocation identity binds these facts:

- planner cohort, schema, policy, and plan identities;
- requirement-profile identity;
- backend class and backend identity;
- adapter and application-host binding identities;
- runtime-authority and effect-permission identities;
- credential-reference and budget identities;
- resource and enforcement-profile identities; and
- persistence and recovery-policy identities.

The store adds the run identity and derives a separate attempt identity before dispatch. Terminal evidence preserves `observed` or `unknown` state for that exact attempt.

## Retry and recovery

Backend drift denies the current attempt. A route change requires a new explicit plan and a new attempt.

An unknown outcome blocks automatic replanning or dispatch. An operator-owned settlement path must resolve uncertainty before another route can be considered.

## Inspection

CLI operation results include the metadata-only backend attempt. TUI run details include the backend class, backend identity, and attempt state.

Safe views omit commands, prompts, credential values, environment values, provider payloads, and physical paths.

## Non-claims

These checks do not prove backend availability, sandbox enforcement, work completion, output correctness, provider correctness, or release readiness.
