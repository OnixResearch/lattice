# Proposal: Webhook listener server

## Summary

Add a first-class webhook listener command that binds an HTTP server to configured addresses, matches persisted webhook triggers, admits requests through the existing trigger core, starts headless workflow runs, and records safe trigger outcomes.

## Problem

The trigger model can validate webhook request facts and start a headless run from an admitted plan, but operators still need an actual listener process that receives HTTP requests, resolves webhook secrets, applies body/content limits, maps requests into trigger events, and returns deterministic HTTP responses. Without a listener, webhook triggers require bespoke glue and cannot be operated as part of n8nclone.

## Goals

- Provide a CLI-managed webhook listener for persisted webhook triggers.
- Keep request parsing, trigger matching, response selection, and admission decisions deterministic and testable without network I/O.
- Resolve webhook credentials only in the listener shell and never expose secrets in diagnostics, history, logs, or responses.
- Start headless workflow runs through the existing trigger plan adapter.
- Record accepted and rejected webhook attempts with safe payload summaries and status information.

## Non-Goals

- Provide a public hosted webhook gateway.
- Add TLS certificate management in the initial listener.
- Add arbitrary queueing or distributed workers.
- Replace the existing scheduler daemon.

## Success Criteria

- `n8nclone webhook-listen` can serve persisted webhook triggers from the configured store.
- Valid requests start exactly one admitted workflow run and persist trigger history.
- Invalid path, method, content type, authentication, body, duplicate idempotency, and disabled-trigger cases return deterministic responses without starting a workflow.
- No plaintext webhook secret or raw sensitive payload appears in logs, history, diagnostics, or response bodies.
