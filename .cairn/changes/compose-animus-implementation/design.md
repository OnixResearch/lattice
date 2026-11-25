# Design

## Context

The selected OMP worker has no child delegation export. Animus owns child-session facts; Lattice owns execution; Kiln owns attempt authority and display.

## Decisions

Pin the launch profile and preserve the existing canonical attempt and candidate ownership. Retain and validate the producer receipt against work, attempt, fence, request digest, incarnation and profile before inspect. Reject unsupported profiles before effects; never replay uncertain execution.

## Risks / Trade-offs

Missing or unverified evidence remains unseen, never proof of zero children or authority to retry.
