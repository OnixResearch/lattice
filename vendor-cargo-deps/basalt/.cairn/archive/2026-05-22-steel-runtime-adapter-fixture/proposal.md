# Proposal: Steel runtime adapter fixture

## Summary

Add a narrow, fixture-backed Steel runtime adapter test seam that proves Basalt can consume real Steel-style evaluation outputs without granting the interpreter ambient authority.

## Problem

Basalt currently defines Steel DTOs and validation, but it does not prove the boundary against an interpreter adapter or fixture that looks like actual Steel execution. That leaves the runtime integration contract under-specified for downstream Steel consumers.

## Scope

- Add an adapter-facing trait or fixture harness that consumes Steel evaluation DTOs and returns structured receipts.
- Include deny-by-default fixtures for unavailable host functions, undeclared authority, malformed outputs, and source-hash mismatches.
- Keep Rust as the sole authority validator and receipt checker.

## Non-goals

- No general-purpose embedded Lisp platform inside Basalt.
- No filesystem/process/network host functions.
- No mutation or orchestration engine.
