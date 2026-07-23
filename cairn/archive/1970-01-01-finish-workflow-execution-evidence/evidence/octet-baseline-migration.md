# Octet baseline migration

The reviewed workflow-execution implementation changes only two pre-existing Executor finding identities:

- `excessive_file_length` changed from 1879 to 1955 source lines.
- `path_segment_repetition` retained the same file, item, lint, group, severity, and message while its source line moved.

The bounded migration normalized only dynamic line-count text for structural findings and compared exact semantic keys for all 1161 full findings. It proved:

- old full findings: 1161
- candidate full findings: 1161
- semantically matched: 1161
- unmatched old: 0
- unmatched candidate: 0
- package accepted/current after migration: 911 / 911
- package new/stale after migration: 0 / 0
- old baseline file BLAKE3: `bb68d531713e860de0d5dbf6dd9a6a34fd4ed41df05af4b5a5f5bb0d2d79473a`
- new baseline file BLAKE3: `849be9da522acceb00f9e02de8607d89251aa8bcfc034d018a53b03741baa54f`

Candidate-only semantic keys remain unconditionally rejected. The older one-finding resolution allowlist remains present in the reusable migration proof but was not exercised: this migration had no unmatched findings.
