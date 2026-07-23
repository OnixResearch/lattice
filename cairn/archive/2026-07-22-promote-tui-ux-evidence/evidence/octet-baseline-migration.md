# Octet baseline migration

Promotion markers and final traceability validation shifted source identities without changing the accepted semantic finding set. Before migration, the remaining TUI cache write that shared an Octet stable identity with another ignored-result finding was changed to report a bounded `Cache update failed` diagnostic instead of discarding its result.

## Proven transformation

- old findings: 1159
- candidate findings: 1159
- semantically matched findings: 1159
- unmatched old findings: 0
- unmatched candidate findings: 0
- old baseline hash: `b3:93d082c36d07531f05d9c5143e24548e358f7223ed1b69254bb8deb4abb848f3`
- new baseline hash: `b3:41adfd53108109da0bcf82948ba8e505647705c3c1d02843f84f205387ea7a98`
- new baseline file BLAKE3: `91a0fc1041049746bef9df7fd769577fd5b6f832c6765d5b6eb1098c1f64154d`
- migration script BLAKE3: `5bd1cdeeda811cbb532629e08dd90eb460387c9035159fc90f490e2bcef65976`

The migration script accepts only exact normalized semantic-set equality. Its positive self-tests cover crate-identity and bounded line-count normalization; its negative tests reject semantic drift and any resolved finding. A dry run reported `old=1159 candidate=1159 matched=1159 unmatched_old=0 unmatched_candidate=0`, execution wrote candidate stable identities, and `octet-precommit-check` then passed.

This is a bounded identity migration, not a waiver, finding addition, finding removal, or broader rebaseline.
