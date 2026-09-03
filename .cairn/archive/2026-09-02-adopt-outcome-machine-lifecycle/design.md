## Context

Lattice owns workflow orchestration and external effects. Agent output remains an input to repository-owned admission and execution decisions.

The repository lacked a pinned Cairn package. Operators depended on an ambient sibling checkout for lifecycle commands.

## Decisions

### Decision: Add a repository-owned Cairn runner

**Choice:** Pin Cairn revision `5e1fe08d62149b286ff583eb056e655fe92e3cf1` and inject its generated policy for lifecycle commands.

**Rationale:** Lattice can reproduce profile selection without ambient sibling state.

### Decision: Preserve Lattice authority

**Choice:** Keep orchestration, effect, evidence, and release meaning inside Lattice checks and maintainers.

**Rationale:** Cairn validates review structure and evidence identity. It does not judge workflow value or external effect truth.

### Decision: Retain negative lifecycle evidence

**Choice:** Store the planned-state sync rejection before attaching completion evidence.

**Rationale:** The pilot must show that delivery pressure cannot bypass repeatability evidence.

## Risks / Trade-offs

- The profile does not prove workflow correctness or external effect truth.
- The new Nix input increases the reviewed dependency surface.
- Fresh-state success does not establish production readiness.
