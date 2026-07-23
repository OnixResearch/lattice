# Migrate workflow-format Octet baseline

## Why

Workflow-format completion shifted accepted finding identities and resolved one assertion-density finding, leaving Octet unable to distinguish reviewed identity drift from semantic finding drift.

## What changes

Migrate the baseline only when dynamic diagnostic counts normalize, no candidate finding is new, and the sole resolved finding exactly names `workflow_diagnostic_from_runner` in `src/workflow_validation.rs`.

## Non-goals

This does not suppress findings, broaden accepted semantic changes, change Octet policy, or claim the accepted findings are fixed.
