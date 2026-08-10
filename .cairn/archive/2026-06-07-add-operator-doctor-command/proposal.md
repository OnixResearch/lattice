# Proposal: Operator doctor command

## Summary

Add an operator-facing `doctor` command that checks configuration, store reachability, Nickel workflow parsing, credential provider readiness, trigger/listener configuration, and example drift with stable text and JSON output.

## Problem

n8nclone now spans workflow storage, Nickel import/export, credential providers, trigger admission, webhook listeners, and headless execution. When an operator cannot run a workflow, there is no single command that distinguishes configuration mistakes, store problems, credential-provider failures, malformed workflows, listener readiness issues, or example drift. Troubleshooting requires ad hoc commands and can accidentally expose sensitive context.

## Goals

- Provide a `doctor` command with stable check identifiers, severities, and safe messages.
- Keep diagnosis decisions in a pure core that accepts collected facts and returns findings.
- Keep filesystem, store, environment, credential-provider, and network bind probes in a thin shell.
- Support human-readable text output and deterministic JSON output.
- Include positive and negative tests for healthy config, missing config, store failures, Nickel parse errors, credential provider failures, and webhook listener readiness issues.

## Non-Goals

- Repair configuration automatically.
- Start workflows or listeners as part of diagnosis.
- Print plaintext secrets, environment values, credential payloads, or raw request bodies.
- Replace focused validation, import, or listener tests.

## Success Criteria

- `n8nclone doctor` reports safe actionable findings for common operator failure modes.
- JSON output is deterministic enough for automation and CI.
- Healthy configurations produce a successful doctor result.
- Fatal findings produce a failing doctor result without mutating workflow state.
- Secret material is redacted or omitted in all doctor output.
