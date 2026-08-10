# Audit remaining TUI/UX traceability

## Why

Tracey now reports exactly 98 missing requirements, all in the accepted TUI/UX specification. Existing implementation and tests cover many of them, but direct evidence strength has not been classified and several broad or adversarial scenarios remain incomplete.

## What changes

Create a byte-bound audit of all 98 IDs against bounded maintained snapshots, classify each row as exact or partial, and preserve the 98-item missing set without attaching markers or changing behavior.

## Non-goals

This audit does not complete TUI behavior, promote markers, remove debt, claim complete terminal compatibility, or make Tracey coverage successful.
