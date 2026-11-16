## Why

Lattice #24 and #13 require configuration, owned execution, physical allocation and inference attribution to remain separate. The current Aspen #42 and Animus #17 proposals do not yet supply consumer-ready lease/readiness/request-admission APIs. Advertising configuration as live capacity or inventing producer DTOs would violate that boundary.

## What Changes

- Project bounded material-free compatibility facts from existing Pi/Clankers/Animus configuration and existing pure planners.
- Reuse the exact admitted runtime identity in host-generated response metadata and optional durable provider usage provenance.
- Preserve unknown producer capability and conservative settlement, reject malformed/crossed configuration provenance, and retain immutable usage through replay/restart.
- Record exact source pins and the remaining producer/live qualification prerequisites without changing sibling sources, pins, scheduling or authority.

## Impact

Agent runtime/profile modules, orchestration usage normalization/validation, focused deterministic fixtures and docs/orchestration-budget-usage.md. Existing receipt persistence already freezes the new optional content; no table migration or alternate store is required. Candidate-worker confinement, implementation-attempt admission and producer implementation remain outside this change. This does not close #24/#13 or establish live capacity or measured command-worker telemetry.
