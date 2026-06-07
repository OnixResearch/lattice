# Proposal: External triggers and webhooks

## Summary

Add durable trigger definitions for manual, scheduled, and webhook-driven workflow starts. Triggers should feed typed payload values into workflows, enforce admission and credential policy before execution, and reuse the existing headless/scheduler runner surfaces.

## Problem

The scheduler currently models recurring due runs, and headless execution can run a named workflow, but workflows cannot be started by explicit trigger definitions or inbound webhook events. Operators need stable trigger identities, typed payload binding, overlap/idempotency policy, and safe authentication for webhook entrypoints.

## Goals

- Persist trigger definitions that bind external events to saved workflows.
- Support manual, recurring schedule, and HTTP webhook trigger kinds behind a common trigger admission model.
- Convert inbound trigger payloads into typed workflow input values.
- Require explicit authentication policy for public webhook triggers.
- Apply overlap, idempotency, and missed-run behavior before starting a workflow.
- Preserve trigger configuration in typed Nickel import/export.

## Non-Goals

- Build a public multi-tenant hosted webhook service.
- Add arbitrary remote worker distribution.
- Replace the existing scheduler daemon in one step.
- Store plaintext webhook secrets in workflow files.

## Success Criteria

- Trigger definitions can be saved, validated, listed, enabled, disabled, imported, and exported.
- A valid webhook request starts the expected workflow with typed payload inputs.
- Invalid signatures, unsupported methods, malformed payloads, disabled triggers, and duplicate idempotency keys fail without starting workflow side effects.
- Scheduled triggers remain compatible with current schedule behavior.
