# Design: Asynchronous Mantle checks

## Context

Lattice owns graph-native orchestration and external adapters. GitHub, Basalt, and Mantle each expose different authority and lifecycle models. The workflow must make those boundaries explicit:

```text
GitHub Actions OIDC endpoint
  -> Lattice acquisition shell
  -> Basalt verification and UCAN exchange
  -> Lattice concrete request planner
  -> Mantle durable attempt
  -> signed Mantle events
  -> Lattice state reducer
  -> GitHub Check Run updates
```

## Functional core

Pure functions own:

- matrix expansion and stable matrix-job identity;
- OIDC audience request planning;
- requested Mantle capability planning;
- concrete build request construction;
- idempotency-key construction;
- event signature and sequence admission inputs;
- workflow and Check Run state reduction;
- bounded annotation and summary projection;
- retry, cancellation, and reconciliation decisions.

The core receives explicit provider observations, time, and policy. It does not read environment variables, call HTTP APIs, read secrets, execute builds, or mutate workflow state.

Stable stack-owned identities use canonical encoding and BLAKE3.

## GitHub OIDC acquisition shell

Inside GitHub Actions, the adapter reads only the documented request URL and request-token variables after the workflow explicitly enables identity-token permission. It adds the configured Basalt audience, enforces HTTPS and allowed host policy, bounds redirects and response bytes, and rejects malformed responses.

The GitHub request token and returned OIDC token remain transient secrecy-aware values. They are never added to workflow inputs, persisted state, logs, annotations, Check Runs, receipts, or error text.

Lattice sends the OIDC token plus the planned capability request to Basalt. It accepts only a UCAN whose public audience, resource scope, operations, and expiry fit the plan. Mantle performs authoritative UCAN verification again when the request arrives.

## SecretSpec credential path

GitHub API credentials, such as an installation token when the selected deployment requires one, are requested through Lattice's credential registry. After `adopt-animus-secretspec-sops-providers` archives, that registry delegates provider resolution to the Animus SecretSpec Rust SDK host adapter.

The workflow stores only a credential reference and public provider metadata. It never stores a provider value or value-derived hash. OIDC tokens and exchanged UCANs use transient transport containers and are not SecretSpec declarations.

## Submission and matrix fan-out

Each matrix job has a canonical identity derived from workflow identity, node identity, matrix coordinates, and attempt generation. The planner creates one concrete Mantle request and least-authority capability request per job.

Submission uses a stable idempotency key. A duplicate call can recover the same attempt only when the authenticated subject and canonical request match. Lattice persists public attempt identity, event cursor, Check Run identity, state version, and evidence references.

Lattice does not send evaluator expressions, arbitrary commands, or secret environment values as Mantle requests. Source and derivation inputs follow the admitted artifact boundary.

### Admitted private producer prerequisite

The implemented Mantle transport consumes `mantle-build-contract::async_api` from the separately admitted `serve-private-asynchronous-build-api` producer change. It uses a bounded private Unix-domain socket with an exact peer UID, configured service/project/holder identities, holder-bound UCAN presentation, durable replay admission, and pinned producer signing keys. This is the real receiver/coordinator/worker path, not an invented HTTP Build endpoint or a substitute authorization check in Lattice. GitHub OIDC and Check Runs retain their independently bounded HTTPS transports.

Producer acceptance requires an immutable validated revision and actual positive/negative native receiver and worker evidence. The broader `add-nix-remote-service-gateway` remains active unless its own gates independently close; private Build API evidence does not prove Nix compatibility, public service exposure, anonymous source availability, deployed authority, or the external GitHub stages. A local signed native smoke is bounded local behavior evidence only. Private-repository and report-only rollout still require separately provisioned real GitHub/OIDC/service authority and their observed evidence before required status.

## Event consumption and state reduction

The shell fetches bounded Mantle event pages by authenticated cursor. It verifies the configured producer key and gives parsed event facts to the reducer.

The reducer enforces event identity, attempt identity, monotonic sequence, allowed transition, and terminal-state immutability. Duplicate events are no-ops. A gap causes bounded reconciliation through Mantle status before later events apply. A terminal Check Run update is idempotent.

Polling and webhook delivery are both shells over the same reducer. Neither transport is state authority.

## GitHub Check Runs

Lattice creates one Check Run per configured workflow or matrix job. It maps admitted, queued, running, terminal success, terminal failure, cancellation, and infrastructure outcomes through a versioned projection policy.

Annotations, title, summary, and text have named count and byte limits. Long logs remain in Mantle and appear as authorized links or evidence references. Lattice does not copy unbounded logs into GitHub.

GitHub rate limits, abuse responses, transient errors, duplicate creates, stale update versions, missing repositories, revoked credentials, and permission denial have explicit retry or terminal policy. Retry delays and counts are named configuration values and use injected time.

## Cancellation and recovery

GitHub cancellation or supersession creates a typed cancellation intent. Lattice requests Mantle cancellation only with owned-attempt authority. Mantle decides final execution state.

On restart, Lattice loads public orchestration state, validates its version, reacquires fresh credentials, reconciles Mantle status, and resumes event consumption. It never persists OIDC, UCAN, or GitHub bearer tokens for recovery.

## Evidence and non-claims

Valence links workflow, matrix job, exchange decision, Mantle attempt, event, result, and Check Run identities. Receipts record bounded public observations and redacted error categories.

A successful Check Run records observed orchestration and Mantle result status. It does not prove build correctness, hermeticity, host isolation, GitHub identity beyond Basalt's checked claims, or release eligibility.

## Rollout

Offline fixtures precede a private GitHub test repository. A selected workflow then runs in report-only mode without required-Check status. Required status is enabled only after recovery, duplicate-event, cancellation, and credential-redaction evidence passes Cairn gates.
