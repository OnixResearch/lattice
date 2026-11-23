## Why

Kiln's established native cancellation rail accepted an exact cancel but observed DomainFailure and a Failed node. The workflow-exchange worker discards the owned process runner's typed cancellation disposition and always supplies `cancelled: false`. A request flag alone cannot repair this: completed success or failure must win a late-cancel race.

## What Changes

- Retain the typed owned process cancellation observation across credential dispatch, executor and snapshot completion into the existing pure exchange completion decision.
- Preserve observed success/failure when cancellation intent has not stopped execution; prevent pending dependent effects after active cancellation.
- Add real local application regressions and document the producer operator contract without changing wire identities or consumer classification.
- Restore exact declared Tile, shared component-runtime and Mantle-verifier dependencies so the existing application integrations and cancellation regressions compile; preserve their behavior and authority boundaries.

## Impact

- **Files:** process credential recipe, executor, headless snapshot shell, workflow-exchange worker/tests, operator documentation and this producer-local lifecycle.
- **Testing:** focused real workflow-exchange regressions, ordinary exit-as-data regression, producer repository and Cairn checks, then Kiln's unchanged native rail against an immutable measured producer.
- **Non-goals:** no consumer relabeling, remote cancellation claims, protocol migration, deployment activation, publication, acceptance or archive.
