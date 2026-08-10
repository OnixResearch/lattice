# Finish workflow-validation evidence

## Why

The workflow-validation audit found seven partial rows and three rows without exact evidence.

## What changes

- Add maintained positive and negative assertions for credential, generic runner, provider, subworkflow, and TUI validation boundaries.
- Add deterministic unsupported runner-policy admission where the selected runtime lacks a declared effect.
- Reclassify all 29 reviewed rows as exact while keeping Tracey coverage unchanged.

## Non-goals

This change does not add Tracey markers, claim arbitrary n8n parity, or establish release readiness.
