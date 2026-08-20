# Proposal: Publish a bounded workflow exchange transport

## Problem

The workflow contract defines requests and observations, but it defines no transport service. Consumers cannot connect a concrete `WorkflowExchangePort` without inventing framing, limits, error classes, or disconnect behavior.

## Outcome

Publish a versioned Unix-domain-socket transport crate beside `lattice-workflow-contract`. The crate will own typed exchange envelopes, deterministic BLAKE3 identities, bounded framing, pure admission, a finite server shell, and a matching client shell.

The transport will call an explicit application-owned workflow handler. It will not select operations, execute workflows, read credentials, or grant authority.

## Scope

- Add an independently consumable `lattice-workflow-transport` crate in the nested workflow workspace.
- Add typed dispatch, observe, control, log-read, and reconciliation envelopes.
- Add deterministic request and response identities.
- Add bounded Unix socket client and server shells.
- Preserve disconnect ambiguity at the client boundary.
- Add positive and negative protocol, framing, peer, timeout, and limit tests.
- Add strict Octet, Clippy, format, and Nix checks.
- Publish producer fixtures and integration guidance for Kiln.

## Exclusions

- No TCP or HTTP endpoint.
- No ambient process execution.
- No operation-catalog selection.
- No workflow runtime implementation inside the transport crate.
- No claim of deployed availability, workflow execution, peer trust, or external effect success.
