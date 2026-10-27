# Clarify policy artifact consumption

## Why

The repository has Nickel policy source and an ignored generated JSON artifact. Downstream applications need a deterministic answer for whether they consume generated policy JSON, generate it themselves, or embed a sample.

## What Changes

- Define the contract for Nickel-authored policy source versus generated JSON consumed by Rust applications.
- Ensure sample policy artifacts are either packaged intentionally or documented as generated local artifacts.
- Add freshness checks so generated policy drift is caught before release.

## Scope

In scope: policy artifact contract, package include/exclude decisions, README/docs, Nickel export freshness gate. Out of scope: changing policy schema semantics beyond what is needed to document consumption.

## Verification

Validate this planning package with Cairn proposal/design/tasks gates. Implementation must run the focused checks named in tasks plus the repo release rail before archive.
