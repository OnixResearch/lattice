# Change: Orchestrate asynchronous Mantle checks

## Why

GitHub Actions can supply short-lived OIDC identity, and Mantle can execute durable concrete build requests. Lattice should connect those systems as a workflow orchestrator. It should not own machine selection, KVM execution, store truth, or build-result authority.

An asynchronous Check Run flow avoids holding a CI process open for long builds. It also gives each matrix job a stable identity, bounded progress, recoverable event handling, and evidence links.

## What Changes

- Add a GitHub Actions transport adapter that requests an OIDC token for an explicit Basalt audience and immediately exchanges it for a short-lived Mantle UCAN.
- Add pure planning for deterministic matrix expansion, concrete Mantle submissions, idempotency keys, and least-authority capability requests.
- Submit builds through the versioned Mantle Build API and persist only public orchestration state.
- Consume signed Mantle events by cursor and reduce them to idempotent workflow and GitHub Check Run transitions.
- Create and update asynchronous GitHub Check Runs with bounded annotations, summaries, and log links.
- Resolve GitHub API credentials through the accepted Animus SecretSpec credential registry path.
- Link workflow, matrix job, Mantle attempt, Check Run, result, and evidence identities through Valence.
- Add positive and negative fixtures for OIDC, exchange, submission, duplicate events, missed events, retries, GitHub failures, cancellation, redaction, and recovery.

## Non-Goals

- OIDC claim verification or UCAN issuance, which belongs to Basalt.
- Build execution, store mutation, machine selection, OOM retry, accounting, or result-sharing policy, which belongs to Mantle.
- Deployment inventory, which belongs to OnixOS.
- Long-lived GitHub or Mantle bearer tokens in workflow definitions.
- Synchronous waiting for every remote build inside a GitHub runner.

## Dependencies

- `adopt-animus-secretspec-sops-providers`, after the Animus prerequisite is archived at an immutable revision.
- Basalt `exchange-oidc-for-ucan-capabilities` at an immutable validated revision.
- Mantle `serve-private-asynchronous-build-api` at an immutable validated revision for the authenticated private Build API. The broader `add-nix-remote-service-gateway` remains a separate prerequisite for any Nix-compatible or public gateway claim and is not completed by this consumer.
- The accepted Valence build-service evidence profile.

## Impact

- **Affected specs:** `github-ci`
- **Affected code:** GitHub transport adapter, OIDC acquisition shell, Basalt exchange client, Mantle client, event reducer, Check Run adapter, workflow state, receipts, and fixtures
