# Proposal: Credential vault providers

## Summary

Add concrete credential-provider backends and optional local vault storage while preserving the existing opaque secret-reference model. Workflows continue to store references, not plaintext secrets; execution resolves secrets through explicitly configured providers with auditable diagnostics and redaction.

## Problem

The credential model already prevents plaintext workflow serialization, but users need practical ways to resolve references in development and production. Without built-in provider configuration, every execution environment must supply custom code or tests. A local encrypted vault and common read-only providers would make workflows runnable while keeping secrets outside workflow files.

## Goals

- Provide provider registry configuration for environment, local encrypted vault, and external command providers.
- Keep provider behavior behind the credential-provider trait and runner/executor boundaries.
- Store local vault entries encrypted at rest with explicit key-source configuration.
- Validate provider names, key shapes, missing secrets, and provider capability before execution.
- Preserve redaction in output, diagnostics, run history, replay bundles, and mapping reports.

## Non-Goals

- Build an enterprise secrets manager.
- Store plaintext secret values in Nickel workflows or run history.
- Auto-migrate secrets from n8n or other tools without explicit operator action.
- Require a local vault for deployments that use external providers.

## Success Criteria

- Workflows can resolve credential references through configured environment, local vault, or external command providers.
- Missing, malformed, unauthorized, or unsupported provider references fail before side-effecting node execution.
- Local vault export/import omits plaintext by default and verifies integrity with BLAKE3 content metadata where applicable.
- TUI/CLI surfaces can test provider reachability without displaying secret values.
