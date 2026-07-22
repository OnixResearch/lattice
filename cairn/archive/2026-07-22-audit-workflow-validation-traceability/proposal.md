# Audit workflow-validation traceability

## Why

Twenty-nine workflow-validation requirements remain unreviewed. Existing tests must be classified without manufacturing Tracey coverage.

## What changes

- Review every missing `workflow-validation` row against maintained tests.
- Bind each decision to safe relative paths and current BLAKE3 file hashes.
- Prove coverage remains unchanged at 290 missing with zero dangling markers.

## Non-goals

This audit does not add requirement markers, complete partial behavior, claim arbitrary n8n compatibility, or establish release readiness.
