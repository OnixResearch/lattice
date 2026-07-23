# Octet baseline migration

The bounded TUI/UX completion changed source locations for existing findings and removed one previously accepted ignored-result finding by handling the background-run completion send explicitly.

## Proven transformation

- old findings: 1161
- candidate findings: 1160
- semantically matched findings: 1159
- candidate-only semantic findings: 0
- allowlisted resolved semantic finding: 1
- old baseline hash: `b3:923eb4e2656647be66cab37000b4eb67a46e45fbfaed247df9c9dc3dc715d629`
- new baseline hash: `b3:93d082c36d07531f05d9c5143e24548e358f7223ed1b69254bb8deb4abb848f3`
- new baseline file BLAKE3: `f8f9ba9919903a4484954e8a9fe189213560efb208e5c7bf3099d7efd5366751`

The exact resolved semantic key is the count reduction from two to one for `ignored_result` in `src/tui/app.rs` with message `` `Result` value silently discarded with `let _ = ...` ``. The migration rejects every candidate-only semantic key and any resolved key outside that exact lint, file, and message allowlist. File-length and source-location identity drift is normalized only for semantic-set comparison; candidate stable IDs remain the written baseline identities.

The migration script passed its positive and negative self-tests, a dry run reported `matched=1159 unmatched_old=1 unmatched_candidate=0`, execution wrote the candidate baseline, and `octet-precommit-check` then passed. This is a bounded identity migration and finding removal, not a waiver or broader rebaseline.
