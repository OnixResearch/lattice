# Tasks

- [x] [serial] Add typed handler profiles, exact route admission, deterministic identities, and positive and negative Nickel fixtures. r[lattice.workflow_interchange.application_profile]
- [x] [serial] Add durable request, idempotency, acceptance, execution, node, and bounded-log storage. r[lattice.workflow_interchange.application_admission]
- [x] [serial] Bind route-exact authority and cancellation to the existing headless runtime. r[lattice.workflow_interchange.application_execution]
- [x] [serial] Implement dispatch, exact acceptance recovery, observe, reconcile, controls, and log reads. r[lattice.workflow_interchange.application_lifecycle] r[lattice.workflow_interchange.application_recovery] r[lattice.workflow_interchange.application_logs]
- [x] [parallel] Add positive tests for dispatch, duplicate admission, recovery, execution, cancellation, logs, reconciliation, and Unix composition. r[lattice.workflow_interchange.application_verification]
- [x] [parallel] Add negative tests for profile drift, idempotency conflict, stale controls, unsupported pause, missing logs, missing records, active limits, and restart uncertainty. r[lattice.workflow_interchange.application_verification]
- [x] [serial] Add `workflow-exchange serve`, documentation, and operator fixtures. r[lattice.workflow_interchange.application_command]
- [x] [serial] Run focused tests, workspace tests, strict Clippy, Octet, Cairn, Nickel, operator checks, and Nix checks. r[lattice.workflow_interchange.application_verification]
- [x] [serial] Record evidence, sync the accepted spec, archive the change, and publish it. r[lattice.workflow_interchange.application_verification]
