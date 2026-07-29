# Tasks

## 1. Confirm prerequisites and baseline

- [ ] [serial] 1.1 Run existing workflow, credential, event, adapter, and GitHub-related tests before core changes. r[lattice.github_ci.evidence_boundary]
- [ ] [serial] 1.2 Confirm archived immutable revisions for the Animus SecretSpec adapter, Basalt exchange, Mantle Build API, and Valence evidence profile. r[lattice.github_ci.secretspec_github_credentials] r[lattice.github_ci.evidence_boundary]
- [ ] [serial] 1.3 Define named OIDC, HTTP, matrix, event-page, cursor, annotation, summary, retry, and recovery bounds. r[lattice.github_ci.oidc_acquisition] r[lattice.github_ci.bounded_check_runs]
- [ ] [serial] 1.4 Define explicit non-claims and authority boundaries. r[lattice.github_ci.evidence_boundary]

## 2. Build pure orchestration cores

- [ ] [serial] 2.1 Add deterministic matrix-job identity and concrete Mantle request planning. r[lattice.github_ci.pure_job_planning]
- [ ] [serial] 2.2 Add least-authority OIDC audience and UCAN capability request planning. r[lattice.github_ci.attenuated_exchange]
- [ ] [serial] 2.3 Add stable submission idempotency keys. r[lattice.github_ci.durable_async_submission]
- [ ] [serial] 2.4 Add the signed-event and Check Run state reducer with sequence and terminal-state rules. r[lattice.github_ci.idempotent_event_reducer]
- [ ] [serial] 2.5 Add positive tests and negative identity, transition, sequence, scope, duplicate, gap, overflow, and terminal-mutation tests. r[lattice.github_ci.idempotent_event_reducer]
- [ ] [serial] 2.6 Add property tests for reducer determinism and duplicate-event idempotency. r[lattice.github_ci.idempotent_event_reducer]

## 3. Add GitHub OIDC and Basalt shells

- [ ] [serial] 3.1 Acquire GitHub OIDC through documented variables under explicit HTTPS, host, redirect, timeout, and response bounds. r[lattice.github_ci.oidc_acquisition]
- [ ] [serial] 3.2 Keep request tokens and OIDC values in transient secrecy-aware containers. r[lattice.github_ci.evidence_boundary]
- [ ] [serial] 3.3 Exchange OIDC and planned capability requests with Basalt. r[lattice.github_ci.attenuated_exchange]
- [ ] [serial] 3.4 Reject returned UCAN facts that exceed the planned audience, operations, resource scope, or expiry. r[lattice.github_ci.attenuated_exchange]
- [ ] [serial] 3.5 Add positive fixtures and negative missing-permission, missing-variable, URL, TLS, timeout, malformed-response, exchange-denial, and overbroad-UCAN fixtures. r[lattice.github_ci.oidc_acquisition] r[lattice.github_ci.attenuated_exchange]

## 4. Add Mantle submission and event clients

- [ ] [serial] 4.1 Submit concrete requests with idempotency keys and persist only public attempt state. r[lattice.github_ci.durable_async_submission]
- [ ] [serial] 4.2 Read bounded signed event pages and verify producer identity. r[lattice.github_ci.idempotent_event_reducer]
- [ ] [serial] 4.3 Reconcile event gaps through bounded status requests. r[lattice.github_ci.idempotent_event_reducer]
- [ ] [serial] 4.4 Add cancellation and restart recovery. r[lattice.github_ci.durable_async_submission]
- [ ] [serial] 4.5 Add positive lifecycle tests and negative unauthorized, duplicate-submit, bad-signature, wrong-attempt, gap, stale-cursor, disconnect, cancellation-race, and malformed-state tests. r[lattice.github_ci.durable_async_submission] r[lattice.github_ci.idempotent_event_reducer]

## 5. Add asynchronous Check Runs

- [ ] [serial] 5.1 Resolve GitHub API credentials through the accepted Lattice credential registry and Animus SecretSpec adapter. r[lattice.github_ci.secretspec_github_credentials]
- [ ] [serial] 5.2 Create and update Check Runs idempotently from reducer outputs. r[lattice.github_ci.bounded_check_runs]
- [ ] [serial] 5.3 Enforce named annotation and summary bounds and link long logs instead of copying them. r[lattice.github_ci.bounded_check_runs]
- [ ] [serial] 5.4 Handle rate limit, abuse response, transient failure, duplicate create, stale update, missing repository, revoked credential, and permission denial. r[lattice.github_ci.bounded_check_runs]
- [ ] [serial] 5.5 Add positive Check Run fixtures and negative API and redaction fixtures. r[lattice.github_ci.bounded_check_runs] r[lattice.github_ci.evidence_boundary]

## 6. Evidence, rollout, and validation

- [ ] [serial] 6.1 Export Valence links across workflow, matrix job, exchange, Mantle attempt, event, result, and Check Run identities. r[lattice.github_ci.evidence_boundary]
- [ ] [serial] 6.2 Add scans that reject GitHub request tokens, OIDC tokens, UCAN bearer tokens, API credentials, provider values, and value-derived hashes in durable output. r[lattice.github_ci.evidence_boundary]
- [ ] [serial] 6.3 Document permissions, configuration, recovery, cancellation, rate limits, incident response, and rollback. r[lattice.github_ci.bounded_check_runs]
- [ ] [serial] 6.4 Run offline, private-repository, and report-only integration stages before required status. r[lattice.github_ci.bounded_check_runs]
- [ ] [serial] 6.5 Run format, focused tests, workspace tests, clippy, lifecycle checks, and relevant Nix checks. r[lattice.github_ci.evidence_boundary]
- [ ] [serial] 6.6 Run Cairn validation, requirement coverage, design gate, and tasks gate. r[lattice.github_ci.evidence_boundary]
