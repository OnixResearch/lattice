# Design

## Functional core

Runtime-policy admission is a pure comparison between a runner node's declared effect policy and the selected runtime profile. Empty declarations preserve existing behavior; built-in profiles admit only their explicit effect labels. Unsupported declarations produce a deterministic fatal diagnostic.

## Evidence completion

A dedicated maintained test surface covers all audit gaps: malformed provider keys, fatal credential diagnostics before effects, all graph rejection classes, terminal-state immutability, runtime policy rejection, provider delegation, invalid subworkflow mappings, and rendered TUI diagnostics/ready state.

## Invariants

- No requirement marker is added during completion.
- Validation occurs before runtime effects.
- Diagnostic text contains bounded identities, never credential values.
- The 290-ID missing set remains unchanged with zero dangling markers.
